# FreeSaaS

A self-hostable office suite you can run on your own infrastructure:
documents, spreadsheets and presentations that open and save real `.docx`,
`.xlsx` and `.pptx` files, connected to **your** storage (OneDrive,
SharePoint, Google Drive, S3, WebDAV, local disk) and **your** sign-in
(Entra ID, Google, Okta, Keycloak, any OIDC/SAML/LDAP provider).

The editors are the open-source [ArtCraft](https://github.com/storytold)
apps, included as git submodules:

| App | Edits | Upstream |
|---|---|---|
| WordCraft | Documents (.docx) | [storytold/wordcraft](https://github.com/storytold/wordcraft) |
| GridCraft | Spreadsheets (.xlsx) | [storytold/gridcraft](https://github.com/storytold/gridcraft) |
| DeckCraft | Presentations (.pptx) | [storytold/deckcraft](https://github.com/storytold/deckcraft) |

FreeSaaS adds what turns three desktop-class editors into a suite: the hub,
storage and identity connectors, cross-app features, AI connectors and
one-command deployment.

> **Status: early scaffold.** The server, provider interfaces, a local-disk
> storage provider and a dev login work. The editors are not wired into the
> hub yet, and the cloud and SSO providers are planned. See
> [docs/PLAN.md](docs/PLAN.md).

## Quick start

```bash
git clone --recursive https://github.com/B91K/freesaas
cd freesaas
cargo run --bin freesaas -- examples/local-dev.yaml
# open http://127.0.0.1:8080
```

Or with Docker:

```bash
docker compose -f deploy/docker-compose/docker-compose.yml up --build
```

Already cloned without `--recursive`? Run `scripts/setup.sh`.

## Configuration

One file, `freesaas.yaml`, picks the providers:

```yaml
identity: { type: oidc, issuer: https://login.microsoftonline.com/<tenant>/v2.0, ... }
storage:
  - { type: onedrive }
  - { type: s3, id: archive, bucket: company-docs, endpoint: https://minio.internal }
ai: { type: ollama, endpoint: http://ollama.internal:11434 }
```

See [examples/](examples/) for complete setups.

## Repository layout

```
upstream/      editors (git submodules; never edited here)
server/        Rust server: core interfaces, providers, HTTP API
web/           hub UI and editor host pages
deploy/        Docker Compose, Helm (planned)
examples/      ready-made freesaas.yaml files
docs/          vision, architecture, plan, licensing
scripts/       setup and upstream-update helpers
```

## License

FreeSaaS is **free for individuals, non-profits, education, and for
development and testing**. Companies running it in production need a
commercial license. Each release becomes Apache-2.0 four years after it
ships. See [docs/LICENSING.md](docs/LICENSING.md) and [LICENSE](LICENSE).

The editors in `upstream/` keep their own MIT OR Apache-2.0 licenses.

FreeSaaS is not affiliated with or endorsed by Microsoft, Google or ArtCraft.
"Microsoft 365", "Word", "Excel" and "PowerPoint" are trademarks of
Microsoft Corporation.
