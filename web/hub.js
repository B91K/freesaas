import { connect } from './storage.js';
import { SignInRequired } from './cloud.js';

// The editors the hub can launch. `connected` editors open and save through the
// hub (see editors/wordcraft-embed/src/bridge.rs); the others run standalone
// with their own Open and Save until their bridge is written.
const EDITORS = {
  wordcraft: {
    label: 'Documents',
    blurb: 'Word documents (.docx)',
    exts: ['docx', 'docm', 'dotx', 'dotm', 'doc', 'odt', 'rtf', 'md', 'txt', 'html', 'htm'],
    connected: true,
  },
  gridcraft: { label: 'Spreadsheets', blurb: 'Excel workbooks (.xlsx)', exts: ['xlsx', 'xlsm', 'xls', 'csv', 'tsv', 'ods'], connected: false },
  deckcraft: { label: 'Presentations', blurb: 'PowerPoint decks (.pptx)', exts: ['pptx'], connected: false },
};

const $ = (sel) => document.querySelector(sel);
const ext = (name) => name.toLowerCase().split('.').pop();
const basename = (path) => path.split('/').pop();
const dirname = (path) => path.split('/').slice(0, -1).join('/');
const join = (dir, name) => (dir ? `${dir}/${name}` : name);
const editorFor = (name) => Object.keys(EDITORS).find((k) => EDITORS[k].exts.includes(ext(name)));

let ctx; // { mode, user, stores, config }
let store; // the active storage backend
let folder = '';
// The document open in the editor frame.
let session = null; // { app, path, pendingOpen: {name, bytes} | null, dirty }

function el(tag, attrs = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k.startsWith('on')) e.addEventListener(k.slice(2), v);
    else if (v !== false && v != null) e.setAttribute(k, v === true ? '' : v);
  }
  e.append(...children);
  return e;
}

function toast(msg, isError = false) {
  const t = $('#toast');
  t.textContent = msg;
  t.className = isError ? 'show error' : 'show';
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => (t.className = ''), 4000);
}

function size(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

function downloadBytes(name, bytes) {
  const url = URL.createObjectURL(new Blob([bytes]));
  el('a', { href: url, download: name }).click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// ---- Files view -----------------------------------------------------------

function renderApps() {
  const apps = $('#apps');
  apps.replaceChildren();
  for (const [id, e] of Object.entries(EDITORS)) {
    apps.append(
      el('button', { class: 'app', onclick: () => openEditor(id, null) },
        el('strong', {}, e.connected ? `New ${e.label.replace(/s$/, '').toLowerCase()}` : e.label),
        el('span', {}, e.blurb),
        e.connected ? '' : el('em', {}, 'Preview: uses its own Open and Save')),
    );
  }
}

async function refresh() {
  const list = $('#files');
  const crumbs = $('#crumbs');
  crumbs.replaceChildren(el('button', { class: 'link', onclick: () => go('') }, store.label));
  let acc = '';
  for (const part of folder.split('/').filter(Boolean)) {
    acc = join(acc, part);
    const target = acc;
    crumbs.append(' / ', el('button', { class: 'link', onclick: () => go(target) }, part));
  }
  renderAccount();
  let entries;
  try {
    entries = await store.list(folder);
  } catch (e) {
    if (e instanceof SignInRequired) {
      renderAccount();
      list.replaceChildren(el('div', { class: 'signin' },
        el('p', {}, e.message),
        el('button', { class: 'button', onclick: () => store.signIn() }, `Sign in to ${store.label}`)));
      return;
    }
    list.replaceChildren(el('p', { class: 'muted' }, `Couldn't list files: ${e.message}`));
    return;
  }
  if (!entries.length) {
    list.replaceChildren(el('p', { class: 'muted' }, 'No files yet. Create a document or upload one.'));
    return;
  }
  list.replaceChildren(...entries.map((f) => {
    const app = f.kind === 'file' && editorFor(f.name);
    const open = f.kind === 'folder' ? () => go(f.path) : app ? () => openFile(f.path) : null;
    return el('div', { class: 'row' },
      el('button', { class: 'name', disabled: !open, onclick: open, title: open ? 'Open' : 'No editor for this file type' },
        el('span', { class: 'icon', 'aria-hidden': 'true' }, f.kind === 'folder' ? '📁' : app === 'gridcraft' ? '📊' : app === 'deckcraft' ? '📽️' : '📄'),
        f.name),
      el('span', { class: 'meta' }, f.kind === 'folder' ? '' : size(f.size)),
      el('span', { class: 'actions' },
        f.kind === 'file' ? el('button', { class: 'link', onclick: () => download(f.path) }, 'Download') : '',
        el('button', { class: 'link danger', onclick: () => remove(f) }, 'Delete')));
  }));
}

// "Sign out" for a signed-in cloud drive; the user's name or where files live otherwise.
function renderAccount() {
  const who = $('#who');
  if (store.signOut && store.signedIn) {
    who.replaceChildren(`${store.label} · `, el('button', { class: 'link', onclick: () => {
      store.signOut();
      refresh();
    } }, 'Sign out'));
  } else {
    who.textContent = ctx.user ? ctx.user.name : store.signOut ? store.label : 'Files stay in this browser';
  }
}

function go(path) {
  folder = path;
  refresh();
}

async function download(path) {
  try {
    downloadBytes(basename(path), await store.read(path));
  } catch (e) {
    toast(`Couldn't download: ${e.message}`, true);
  }
}

async function remove(f) {
  if (!confirm(`Delete ${f.name}? This can't be undone.`)) return;
  try {
    await store.delete(f.path);
    refresh();
  } catch (e) {
    toast(`Couldn't delete: ${e.message}`, true);
  }
}

async function upload(files) {
  for (const file of files) {
    try {
      await store.write(join(folder, file.name), await file.arrayBuffer());
    } catch (e) {
      toast(`Couldn't upload ${file.name}: ${e.message}`, true);
    }
  }
  refresh();
}

// ---- Editor view ----------------------------------------------------------

async function openFile(path) {
  const app = editorFor(path);
  if (!EDITORS[app].connected) {
    toast(`${EDITORS[app].label} can't open files from here yet. Download the file, then use Open in the app.`);
    return openEditor(app, null);
  }
  try {
    openEditor(app, path, { name: basename(path), bytes: await store.read(path) });
  } catch (e) {
    toast(`Couldn't open: ${e.message}`, true);
  }
}

function openEditor(app, path, pendingOpen = null) {
  // editorName: the name the editor saves this document under, when it
  // differs from the stored file's name.
  session = { app, path, pendingOpen, dirty: false, editorName: null };
  const fresh = !pendingOpen && EDITORS[app].connected;
  $('#frame').src = `editors/${app}/index.html${fresh ? '?new' : ''}`;
  $('#doc-name').textContent = path ? basename(path) : EDITORS[app].connected ? 'New document' : EDITORS[app].label;
  setStatus(EDITORS[app].connected ? (path ? 'Saved' : 'Not saved yet') : 'Use the app\'s own Open and Save');
  document.body.classList.add('editing');
}

function closeEditor() {
  if (session?.dirty && !confirm('Leave without saving your changes?')) return;
  session = null;
  $('#frame').src = 'about:blank';
  document.body.classList.remove('editing');
  refresh();
}

function setStatus(text) {
  $('#doc-status').textContent = text;
}

function sendOpen(name, bytes) {
  $('#frame').contentWindow.postMessage({ source: 'freesaas-hub', type: 'open', name, bytes }, location.origin, [bytes]);
}

async function onSave(name, bytes) {
  const app = EDITORS[session.app];
  // Exports (PDF, pictures) are copies: hand them to the browser.
  if (!app.exts.includes(ext(name))) {
    downloadBytes(name, bytes);
    return;
  }
  setStatus('Saving…');
  try {
    // A save of the open file overwrites it. Anything else (the first save of
    // a new document) gets a free name next to it: the editor names new
    // documents Document1.docx, which mustn't replace an existing file.
    const path = session.path && (name === basename(session.path) || name === session.editorName)
      ? session.path
      : await freePath(session.path ? dirname(session.path) : folder, name);
    await store.write(path, bytes);
    session.path = path;
    session.editorName = name;
    $('#doc-name').textContent = basename(path);
    setStatus(session.dirty ? 'Unsaved changes' : 'Saved');
  } catch (e) {
    setStatus('Not saved');
    toast(`Couldn't save ${name}: ${e.message}`, true);
  }
}

// `name`, or `name (2)`, `name (3)`, … if that is taken in `dir`.
async function freePath(dir, name) {
  const taken = new Set((await store.list(dir)).map((f) => f.name));
  const dot = name.lastIndexOf('.');
  const [stem, suffix] = dot > 0 ? [name.slice(0, dot), name.slice(dot)] : [name, ''];
  let candidate = name;
  for (let n = 2; taken.has(candidate); n++) candidate = `${stem} (${n})${suffix}`;
  return join(dir, candidate);
}

async function onOpenRequest() {
  const dialog = $('#picker');
  const list = $('#picker-files');
  list.replaceChildren(el('p', { class: 'muted' }, 'Loading…'));
  dialog.showModal();
  let entries = [];
  try {
    entries = (await store.list(folder)).filter((f) => f.kind === 'file' && editorFor(f.name) === session.app);
  } catch (e) {
    list.replaceChildren(el('p', { class: 'muted' }, `Couldn't list files: ${e.message}`));
    return;
  }
  list.replaceChildren(...(entries.length ? entries.map((f) =>
    el('button', { class: 'row name', onclick: async () => {
      dialog.close();
      try {
        sendOpen(f.name, await store.read(f.path));
        session.path = f.path;
        session.editorName = null;
        $('#doc-name').textContent = f.name;
        setStatus('Saved');
      } catch (e) {
        toast(`Couldn't open: ${e.message}`, true);
      }
    } }, f.name)) : [el('p', { class: 'muted' }, 'No documents here yet.')]));
}

window.addEventListener('message', (e) => {
  if (e.origin !== location.origin || e.source !== $('#frame').contentWindow || !session) return;
  const msg = e.data;
  if (msg?.source !== 'freesaas-editor' || msg.app !== session.app) return;
  switch (msg.type) {
    case 'ready':
      if (session.pendingOpen) {
        sendOpen(session.pendingOpen.name, session.pendingOpen.bytes);
        session.pendingOpen = null;
      }
      break;
    case 'dirty':
      session.dirty = !!msg.dirty;
      setStatus(msg.dirty ? 'Unsaved changes' : 'Saved');
      break;
    case 'save':
      if (typeof msg.name === 'string' && msg.bytes instanceof ArrayBuffer) onSave(msg.name, msg.bytes);
      break;
    case 'open-request':
      onOpenRequest();
      break;
  }
});

window.addEventListener('beforeunload', (e) => {
  if (session?.dirty) e.preventDefault();
});

// ---- Start ----------------------------------------------------------------

async function main() {
  $('#back').addEventListener('click', closeEditor);
  $('#upload').addEventListener('change', (e) => {
    upload([...e.target.files]);
    e.target.value = '';
  });
  $('#picker-upload').addEventListener('change', async (e) => {
    const [file] = e.target.files;
    e.target.value = '';
    if (!file) return;
    $('#picker').close();
    sendOpen(file.name, await file.arrayBuffer());
    session.path = null;
    session.editorName = null;
    $('#doc-name').textContent = file.name;
    setStatus('Not saved yet');
  });
  renderApps();
  try {
    ctx = await connect();
  } catch (e) {
    $('#files').replaceChildren(el('p', { class: 'muted' }, e.message));
    return;
  }
  store = ctx.signedIn ?? ctx.stores[0];
  if (ctx.signInError) toast(ctx.signInError.message, true);
  const select = $('#store');
  select.replaceChildren(...ctx.stores.map((s, i) => el('option', { value: i, selected: s === store }, s.label)));
  select.hidden = ctx.stores.length < 2;
  select.addEventListener('change', () => {
    store = ctx.stores[select.value];
    go('');
  });
  refresh();
}

main();
