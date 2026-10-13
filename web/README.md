# web

The hub: launcher, file browser and the frame the editors run in. Plain ES
modules, no build step.

- `hub.js`: UI, and the postMessage bridge to the editors (protocol in
  `editors/wordcraft-embed/src/bridge.rs`)
- `storage.js`: storage backends with the same shape as the server's
  `StorageProvider`: `ServerStorage` (the FreeSaaS API) and `BrowserStorage`
  (IndexedDB, for static hosting such as GitHub Pages)
- `config.json` (optional, generated at deploy time): `{"mode": "static"}` or
  `{"mode": "server"}`. Without it the hub uses the server if one answers,
  otherwise browser storage. Everything in it is public: never put secrets
  there.

The editors' builds go in `editors/<app>/`, produced by
`scripts/build-web.sh`.
