# Architecture

```
 Browser                                 FreeSaaS server (one Rust binary)
┌──────────────────────────────┐        ┌───────────────────────────────────┐
│ Hub (web/)                   │  HTTP  │ API  /api/me, /api/storage/...    │
│  launcher · files · sharing  │◄──────►│                                   │
│ ┌──────────────────────────┐ │        │ IdentityProvider ─ oidc│saml│ldap │
│ │ WordCraft / GridCraft /  │ │        │ StorageProvider  ─ onedrive│gdrive│
│ │ DeckCraft  (WASM)        │ │        │                    s3│webdav│local│
│ └──────────────────────────┘ │        │ Interop · AI connector · (WOPI)   │
└──────────────────────────────┘        └───────────────────────────────────┘
```

Editing happens in the browser (WASM), so the server stays light: it
authenticates users, brokers file access to the configured storage, and
serves the hub and editor builds.

## Two ways to run

| | Server mode | Static mode |
|---|---|---|
| Hosting | `freesaas` binary or Docker | Any static host, e.g. GitHub Pages |
| Storage | Providers configured on the server | In the browser (IndexedDB); later OneDrive/Google Drive directly from the browser |
| Sign-in | Identity provider on the server | None yet; later the user's own Microsoft/Google account |
| Secrets | Server-side only | **None, by design.** Everything shipped is public. |

The hub (`web/`) is the same in both: `storage.js` picks `ServerStorage` or
`BrowserStorage` from `config.json`, or by probing for the server.

## Editor bridge

Editors run in an iframe in the hub and talk to it with `postMessage`
(same origin only). WordCraft's bridge is `editors/wordcraft-embed`: a small
crate built on the upstream WordCraft crates, unchanged, that replaces the
browser file picker and downloads with hub messages. The protocol is
documented in `editors/wordcraft-embed/src/bridge.rs`:

- editor → hub: `ready`, `open-request`, `save {name, bytes}`, `dirty`
- hub → editor: `open {name, bytes}`

GridCraft and DeckCraft currently run as upstream builds with their own
Open and Save.

## Crates

| Crate | Path | Role |
|---|---|---|
| `freesaas-core` | `server/core` | `StorageProvider`, `IdentityProvider`, `Config`, errors |
| `freesaas-storage-local` | `server/providers/storage-local` | Local disk, one folder per user. **Reference provider.** |
| `freesaas-identity-dev` | `server/providers/identity-dev` | Single fixed user for local development |
| `freesaas-server` | `server/freesaas-server` | HTTP API, config loading, provider wiring (`freesaas` binary) |

## HTTP API (v0)

| Method | Path | |
|---|---|---|
| GET | `/healthz` | Liveness |
| GET | `/api/me` | Current user |
| GET | `/api/storage` | Configured storage ids |
| GET | `/api/storage/{id}/list?path=` | List a folder |
| GET / PUT / DELETE | `/api/storage/{id}/file?path=` | Read / write / delete a file |

Paths are provider-relative with `/` separators. Providers must reject paths
that escape their root.

## Writing a provider

1. Create `server/providers/<kind>-<name>/` and implement the trait from
   `freesaas-core`. Copy `storage-local` as a starting point.
2. Add a match arm in `server/freesaas-server/src/providers.rs` that builds
   it from its `ProviderConfig` keys.
3. Add an example to `examples/` and tests covering path handling and errors.

## Planned building blocks

- **Editor bridge:** load the upstream WASM builds in the hub and connect
  their JSON control channel to `/api/storage` for open and save.
- **OIDC provider:** one implementation covers Entra ID, Google, Okta,
  Keycloak and Authentik. Sessions go in signed cookies.
- **Cloud storage:** OneDrive/SharePoint through Microsoft Graph, Google
  Drive API, S3, WebDAV. Backends that act on the user's behalf use the
  user's delegated token from the identity provider.
- **WOPI host and client:** lets existing Nextcloud, ownCloud, Seafile or
  SharePoint installations use our editors, and lets us talk to any
  WOPI-capable store.
- **Interop:** cross-app features built on the editors' command and MCP
  interfaces.
- **Real-time collaboration:** last, after single-user editing, sharing,
  comments and versions.

## Upstream policy

- `upstream/*` are shallow git submodules pinned to exact commits. They're
  never edited in this repo.
- `scripts/update-upstream.sh` moves them forward. Each bump is its own
  reviewed commit, merged only when CI and the file-compatibility tests pass.
- When we need a patch, we fork the app under our org, point the submodule at
  the fork, keep the patch small, and open a pull request upstream.
