// Cloud storage reached straight from the browser with the user's own
// account: no server and no secrets. Each backend has the same shape as
// `BrowserStorage` / `ServerStorage` in storage.js, plus sign-in.
//
// Sign-in uses only public values from config.json (OAuth client ids). Tokens
// live in sessionStorage, so they last for this tab and are gone when it
// closes.

const PENDING = 'freesaas.oauth.pending';
const tokenKey = (id) => `freesaas.oauth.token.${id}`;

// Where the provider sends the user back: the hub page itself. It must match
// the redirect URI registered with the provider exactly.
export const redirectUri = () => location.origin + location.pathname;

export class SignInRequired extends Error {
  constructor(store) {
    super(`Sign in to ${store.label} to see your files.`);
    this.store = store;
  }
}

function b64url(bytes) {
  return btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function randomString() {
  return b64url(crypto.getRandomValues(new Uint8Array(32)));
}

function loadToken(id) {
  try {
    return JSON.parse(sessionStorage.getItem(tokenKey(id)));
  } catch {
    return null;
  }
}

function saveToken(id, token) {
  sessionStorage.setItem(tokenKey(id), JSON.stringify(token));
}

function clearToken(id) {
  sessionStorage.removeItem(tokenKey(id));
}

async function errorMessage(r) {
  try {
    const body = await r.json();
    return body.error?.message ?? body.error_description ?? body.error ?? r.statusText;
  } catch {
    return r.statusText || `HTTP ${r.status}`;
  }
}

class CloudStorage {
  get signedIn() {
    return !!loadToken(this.id);
  }

  signOut() {
    clearToken(this.id);
  }

  // Remember what we asked for, then leave for the provider's sign-in page.
  beginSignIn(url, extra = {}) {
    const state = randomString();
    sessionStorage.setItem(PENDING, JSON.stringify({ provider: this.id, state, ...extra }));
    url.searchParams.set('state', state);
    location.assign(url.toString());
  }

  // An authorized request; a rejected token means signing in again.
  async request(url, init = {}) {
    const token = await this.accessToken();
    const r = await fetch(url, { ...init, headers: { ...init.headers, Authorization: `Bearer ${token}` } });
    if (r.status === 401) {
      clearToken(this.id);
      throw new SignInRequired(this);
    }
    if (!r.ok) throw new Error(await errorMessage(r));
    return r;
  }
}

// OneDrive and SharePoint document libraries through Microsoft Graph. Sign-in
// is the authorization-code flow with PKCE for single-page apps: the app
// registration's platform must be "Single-page application".
export class OneDriveStorage extends CloudStorage {
  id = 'onedrive';
  label = 'OneDrive';
  static scopes = 'openid profile offline_access User.Read Files.ReadWrite';

  constructor({ clientId, tenant = 'common' }) {
    super();
    this.clientId = clientId;
    this.authority = `https://login.microsoftonline.com/${encodeURIComponent(tenant)}/oauth2/v2.0`;
  }

  async signIn() {
    const verifier = randomString();
    const challenge = b64url(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier)));
    const url = new URL(`${this.authority}/authorize`);
    url.search = new URLSearchParams({
      client_id: this.clientId,
      response_type: 'code',
      redirect_uri: redirectUri(),
      response_mode: 'query',
      scope: OneDriveStorage.scopes,
      code_challenge: challenge,
      code_challenge_method: 'S256',
      prompt: 'select_account',
    });
    this.beginSignIn(url, { verifier });
  }

  async #tokenRequest(params) {
    const r = await fetch(`${this.authority}/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({ client_id: this.clientId, scope: OneDriveStorage.scopes, ...params }),
    });
    if (!r.ok) throw new Error(`Microsoft sign-in failed: ${await errorMessage(r)}`);
    const t = await r.json();
    saveToken(this.id, { access: t.access_token, refresh: t.refresh_token, expires: Date.now() + (t.expires_in - 60) * 1000 });
  }

  async completeSignIn(params, pending) {
    await this.#tokenRequest({ grant_type: 'authorization_code', code: params.get('code'), redirect_uri: redirectUri(), code_verifier: pending.verifier });
  }

  async accessToken() {
    const t = loadToken(this.id);
    if (!t) throw new SignInRequired(this);
    if (Date.now() < t.expires) return t.access;
    if (!t.refresh) throw new SignInRequired(this);
    try {
      await this.#tokenRequest({ grant_type: 'refresh_token', refresh_token: t.refresh });
    } catch {
      clearToken(this.id);
      throw new SignInRequired(this);
    }
    return loadToken(this.id).access;
  }

  // Graph addresses items by path: /me/drive/root:/a/b.docx
  #item(path) {
    const enc = path.split('/').filter(Boolean).map(encodeURIComponent).join('/');
    return enc ? `https://graph.microsoft.com/v1.0/me/drive/root:/${enc}:` : 'https://graph.microsoft.com/v1.0/me/drive/root';
  }

  async list(folder = '') {
    const out = [];
    let url = `${this.#item(folder)}/children?$top=200&$select=name,size,folder,file,lastModifiedDateTime`;
    while (url) {
      const page = await (await this.request(url)).json();
      for (const i of page.value) {
        out.push({
          path: folder ? `${folder}/${i.name}` : i.name,
          name: i.name,
          kind: i.folder ? 'folder' : 'file',
          size: i.size ?? 0,
          modified: i.lastModifiedDateTime ? Math.floor(Date.parse(i.lastModifiedDateTime) / 1000) : null,
        });
      }
      url = page['@odata.nextLink'];
    }
    return out.sort((a, b) => (a.kind !== b.kind ? (a.kind === 'folder' ? -1 : 1) : a.name.localeCompare(b.name)));
  }

  async read(path) {
    // The item carries a short-lived, pre-authenticated download URL.
    const item = await (await this.request(this.#item(path))).json();
    const url = item['@microsoft.graph.downloadUrl'];
    if (!url) throw new Error(`${path} is not a file`);
    const r = await fetch(url);
    if (!r.ok) throw new Error(await errorMessage(r));
    return r.arrayBuffer();
  }

  async write(path, bytes) {
    // Simple upload: fine up to 250 MB.
    const r = await this.request(`${this.#item(path)}/content`, { method: 'PUT', headers: { 'Content-Type': 'application/octet-stream' }, body: bytes });
    const i = await r.json();
    return { path, name: i.name, kind: 'file', size: i.size, modified: Math.floor(Date.parse(i.lastModifiedDateTime) / 1000) };
  }

  async delete(path) {
    await this.request(this.#item(path), { method: 'DELETE' });
  }
}

// Google Drive. Sign-in is Google's token flow for browser apps (no secret).
// The `drive.file` scope only shows files this app created or the user
// opened with it; seeing the whole Drive needs Google's restricted-scope
// verification.
export class GoogleDriveStorage extends CloudStorage {
  id = 'gdrive';
  label = 'Google Drive';
  static scope = 'https://www.googleapis.com/auth/drive.file';
  static api = 'https://www.googleapis.com/drive/v3/files';
  static upload = 'https://www.googleapis.com/upload/drive/v3/files';

  constructor({ clientId }) {
    super();
    this.clientId = clientId;
  }

  async signIn() {
    const url = new URL('https://accounts.google.com/o/oauth2/v2/auth');
    url.search = new URLSearchParams({
      client_id: this.clientId,
      redirect_uri: redirectUri(),
      response_type: 'token',
      scope: GoogleDriveStorage.scope,
      include_granted_scopes: 'true',
      prompt: 'select_account',
    });
    this.beginSignIn(url);
  }

  async completeSignIn(params) {
    const expiresIn = Number(params.get('expires_in')) || 3600;
    saveToken(this.id, { access: params.get('access_token'), expires: Date.now() + (expiresIn - 60) * 1000 });
  }

  async accessToken() {
    const t = loadToken(this.id);
    // No refresh tokens in the browser flow: an expired token means signing in again.
    if (!t || Date.now() >= t.expires) {
      clearToken(this.id);
      throw new SignInRequired(this);
    }
    return t.access;
  }

  // Drive has ids, not paths. Files are listed flat and addressed by name.
  async #files() {
    const out = [];
    let pageToken = '';
    do {
      const q = new URLSearchParams({
        q: "trashed = false and mimeType != 'application/vnd.google-apps.folder'",
        fields: 'nextPageToken, files(id, name, size, modifiedTime)',
        pageSize: '1000',
        spaces: 'drive',
      });
      if (pageToken) q.set('pageToken', pageToken);
      const page = await (await this.request(`${GoogleDriveStorage.api}?${q}`)).json();
      out.push(...page.files);
      pageToken = page.nextPageToken;
    } while (pageToken);
    return out;
  }

  async #id(name) {
    const f = (await this.#files()).find((f) => f.name === name);
    if (!f) throw new Error(`not found: ${name}`);
    return f.id;
  }

  async list() {
    return (await this.#files())
      .map((f) => ({ path: f.name, name: f.name, kind: 'file', size: Number(f.size ?? 0), modified: f.modifiedTime ? Math.floor(Date.parse(f.modifiedTime) / 1000) : null }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  async read(path) {
    return (await this.request(`${GoogleDriveStorage.api}/${await this.#id(path)}?alt=media`)).arrayBuffer();
  }

  async write(path, bytes) {
    const existing = (await this.#files()).find((f) => f.name === path);
    let r;
    if (existing) {
      r = await this.request(`${GoogleDriveStorage.upload}/${existing.id}?uploadType=media&fields=id,name,size,modifiedTime`, { method: 'PATCH', body: bytes });
    } else {
      const body = new FormData();
      body.append('metadata', new Blob([JSON.stringify({ name: path })], { type: 'application/json' }));
      body.append('file', new Blob([bytes]));
      r = await this.request(`${GoogleDriveStorage.upload}?uploadType=multipart&fields=id,name,size,modifiedTime`, { method: 'POST', body });
    }
    const f = await r.json();
    return { path: f.name, name: f.name, kind: 'file', size: Number(f.size ?? bytes.byteLength), modified: Math.floor(Date.parse(f.modifiedTime) / 1000) };
  }

  async delete(path) {
    await this.request(`${GoogleDriveStorage.api}/${await this.#id(path)}`, { method: 'DELETE' });
  }
}

// Cloud backends named in config.json:
//   "onedrive": {"clientId": "...", "tenant": "common"}
//   "gdrive":   {"clientId": "..."}
export function cloudStores(config) {
  const stores = [];
  if (config.onedrive?.clientId) stores.push(new OneDriveStorage(config.onedrive));
  if (config.gdrive?.clientId) stores.push(new GoogleDriveStorage(config.gdrive));
  return stores;
}

// If this page load is the provider sending the user back, finish signing in,
// clean the URL, and return the store that was signed in to.
export async function completePendingSignIn(stores) {
  let pending;
  try {
    pending = JSON.parse(sessionStorage.getItem(PENDING));
  } catch {
    pending = null;
  }
  if (!pending) return null;
  // Microsoft answers in the query string, Google in the fragment.
  const params = new URLSearchParams(location.search);
  for (const [k, v] of new URLSearchParams(location.hash.slice(1))) params.set(k, v);
  if (!params.has('state')) return null;
  sessionStorage.removeItem(PENDING);
  history.replaceState(null, '', redirectUri());
  const store = stores.find((s) => s.id === pending.provider);
  if (!store || params.get('state') !== pending.state) throw new Error('Sign-in failed: the response did not match the request.');
  if (params.has('error')) throw new Error(`Sign-in was not completed: ${params.get('error_description') ?? params.get('error')}`);
  await store.completeSignIn(params, pending);
  return store;
}
