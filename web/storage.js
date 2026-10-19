import { cloudStores, completePendingSignIn } from './cloud.js';

// Storage backends for the hub. Each one has the same shape as the server's
// `StorageProvider` trait: list / read / write / delete, with `/`-separated
// paths relative to the backend's root.

// Files kept in this browser (IndexedDB). Used when there's no server, e.g. on
// GitHub Pages. Nothing leaves the user's machine.
export class BrowserStorage {
  id = 'browser';
  label = 'This browser';

  #db = null;

  async #open() {
    if (this.#db) return this.#db;
    this.#db = await new Promise((resolve, reject) => {
      const req = indexedDB.open('freesaas', 1);
      req.onupgradeneeded = () => req.result.createObjectStore('files', { keyPath: 'path' });
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
    return this.#db;
  }

  async #tx(mode, fn) {
    const db = await this.#open();
    return new Promise((resolve, reject) => {
      const tx = db.transaction('files', mode);
      const req = fn(tx.objectStore('files'));
      tx.oncomplete = () => resolve(req?.result);
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error ?? new Error('storage transaction aborted'));
    });
  }

  async list() {
    const all = await this.#tx('readonly', (s) => s.getAll());
    return all
      .map(({ path, size, modified }) => ({ path, name: path.split('/').pop(), kind: 'file', size, modified }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  async read(path) {
    const rec = await this.#tx('readonly', (s) => s.get(path));
    if (!rec) throw new Error(`not found: ${path}`);
    return rec.bytes;
  }

  async write(path, bytes) {
    const rec = { path, size: bytes.byteLength, modified: Math.floor(Date.now() / 1000), bytes };
    await this.#tx('readwrite', (s) => s.put(rec));
    return { path, name: path.split('/').pop(), kind: 'file', size: rec.size, modified: rec.modified };
  }

  async delete(path) {
    await this.#tx('readwrite', (s) => s.delete(path));
  }
}

// A storage provider configured on the FreeSaaS server (`/api/storage/{id}`).
export class ServerStorage {
  constructor(id) {
    this.id = id;
    this.label = id;
  }

  #url(op, path = '') {
    return `api/storage/${encodeURIComponent(this.id)}/${op}?path=${encodeURIComponent(path)}`;
  }

  async #fetch(url, init) {
    const r = await fetch(url, init);
    if (!r.ok) {
      let msg = r.statusText;
      try { msg = (await r.json()).error ?? msg; } catch { /* not JSON */ }
      throw new Error(msg);
    }
    return r;
  }

  async list(folder = '') {
    return (await this.#fetch(this.#url('list', folder))).json();
  }

  async read(path) {
    return (await this.#fetch(this.#url('file', path))).arrayBuffer();
  }

  async write(path, bytes) {
    return (await this.#fetch(this.#url('file', path), { method: 'PUT', body: bytes })).json();
  }

  async delete(path) {
    await this.#fetch(this.#url('file', path), { method: 'DELETE' });
  }
}

// Pick backends: `config.json` if the site ships one, else the server's
// configured storage if a server answers, else this browser. Cloud drives
// named in config.json (cloud.js) are added after them.
export async function connect() {
  let config = {};
  try {
    const r = await fetch('config.json', { cache: 'no-store' });
    if (r.ok) config = await r.json();
  } catch { /* no config: auto-detect */ }

  let base = null;
  if (config.mode !== 'static') {
    try {
      const [me, ids] = await Promise.all(['api/me', 'api/storage'].map(async (u) => {
        const r = await fetch(u);
        if (!r.ok) throw new Error(r.statusText);
        return r.json();
      }));
      if (ids.length) base = { mode: 'server', user: me, stores: ids.map((id) => new ServerStorage(id)) };
    } catch {
      if (config.mode === 'server') throw new Error('The FreeSaaS server is not reachable.');
    }
  }
  base ??= { mode: 'static', user: null, stores: [new BrowserStorage()] };

  const stores = [...base.stores, ...cloudStores(config)];
  // Back from a provider's sign-in page: finish, and open that store.
  let signedIn = null;
  let signInError = null;
  try {
    signedIn = await completePendingSignIn(stores);
  } catch (e) {
    signInError = e;
  }
  return { ...base, stores, config, signedIn, signInError };
}
