# web

The hub: launcher, file browser and the frame the editors run in. Plain ES
modules, no build step.

- `hub.js`: UI, and the postMessage bridge to the editors (protocol in
  `editors/wordcraft-embed/src/bridge.rs`)
- `storage.js`: storage backends with the same shape as the server's
  `StorageProvider`: `ServerStorage` (the FreeSaaS API) and `BrowserStorage`
  (IndexedDB, for static hosting such as GitHub Pages)
- `cloud.js`: OneDrive and Google Drive backends, with sign-in done in the
  browser using the user's own account. Setup: `docs/CLOUD-DRIVES.md`.
- `config.json` (optional, generated at deploy time):
  `{"mode": "static" | "server", "onedrive": {"clientId": "…"}, "gdrive": {"clientId": "…"}}`.
  Without `mode`, the hub uses the server if one answers, otherwise browser
  storage. Everything in it is public: never put secrets there.

The editors' builds go in `editors/<app>/`, produced by
`scripts/build-web.sh`.
