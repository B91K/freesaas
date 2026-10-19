# Plan

## Phase 0: Validate (2–3 weeks)
- [x] Repo skeleton, editors as submodules, provider interfaces, local storage, dev login
- [x] Build each editor's WASM target (WordCraft: ~40 MB .wasm, ~8 min release build)
- [x] Hub ↔ editor bridge for WordCraft (`editors/wordcraft-embed`, no upstream fork needed)
- [x] Static mode: hub + editors on GitHub Pages, files in browser storage
- [ ] Bridges for GridCraft and DeckCraft
- [ ] Shrink load time: bundle size, compression, caching, a loading screen
- [ ] File-compatibility corpus: 100+ real .docx/.xlsx/.pptx files, round-trip tests in CI
- [ ] Check text input (IME), accessibility and mobile behaviour of the canvas UI
- [ ] Legal: confirm licensing and the CLA, choose a product name, register a legal entity
- [ ] Contact the ArtCraft team about collaboration

## Phase 1: Usable suite (6–8 weeks)
- [ ] Hub: launcher, file browser, new/open/rename/delete, recent files
- [ ] Editors embedded in the hub, opening and saving through `/api/storage`
- [ ] OIDC identity provider (Entra ID, Google, Keycloak tested)
- [ ] Storage providers: S3, OneDrive/SharePoint, Google Drive, WebDAV
- [x] OneDrive and Google Drive straight from the browser (public client ids, no server)
- [ ] Google Picker, so `drive.file` users can open existing Drive files
- [ ] Docker image published. Docker Compose quick start with bundled Keycloak and MinIO.
- [ ] Public launch: docs site, demo instance, GitHub Sponsors

## Phase 2: A suite, not three apps (3–4 months)
- [ ] Sharing links and permissions, version history, comments
- [ ] Interop: sheet ranges in documents, document → deck, mail merge
- [ ] AI connector: any LLM or local model, driving the editors through MCP
- [ ] SAML and LDAP providers
- [ ] WOPI, for Nextcloud and SharePoint integration

## Phase 3: Enterprise (6+ months)
- [ ] Helm chart, high availability, audit logs, admin console
- [ ] Real-time co-editing
- [ ] Mail, calendar, chat and video by integrating existing open-source projects
- [ ] Commercial offering: licenses, support tiers, migration services

## Risks
| Risk | Mitigation |
|---|---|
| Editors are alpha, very new, and maintained by one team | Contribute upstream, pin versions, run compatibility CI, build a relationship with ArtCraft |
| Office file fidelity | Large real-world test corpus. Report and fix issues upstream. |
| Canvas UI accessibility | Assess in Phase 0. Accessibility is often a hard requirement for public-sector buyers. |
| Trademarks | Never use Microsoft, Office, 365 or ArtCraft branding in our name or logo |
