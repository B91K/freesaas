# Vision

**Anyone should be able to run a complete office suite on their own terms,
without paying per seat for software.**

## The product

A self-hostable suite, comparable to Microsoft 365 for everyday office work,
that a developer can deploy on their own infrastructure in minutes:

- **Editors:** documents, spreadsheets, presentations (WordCraft, GridCraft,
  DeckCraft), working with real Office file formats.
- **Bring your own cloud:** files stay where they already are: OneDrive,
  SharePoint, Google Drive, S3-compatible storage, WebDAV/Nextcloud, or local
  disk. Nobody has to migrate data to try it.
- **Bring your own identity:** sign in with whatever the organization already
  uses: Entra ID, Google, Okta, Keycloak, Authentik, any OIDC or SAML
  provider, or LDAP.
- **Inter-app connectivity:** the apps work as one suite: embed a live sheet
  range in a document, build a deck from a document outline, mail-merge from
  a spreadsheet.
- **AI-native, provider-neutral:** every editor exposes its commands to AI
  agents (MCP). Plug in any model, including local models for full privacy.

## What we build vs. what we reuse

We **reuse** the editors and, later, existing open-source mail, chat and
video projects. We **build** the parts that turn them into a suite: hub,
connectors, cross-app features and deployment. We send editor fixes upstream
rather than maintaining a fork.

## Business model

- **Free** for individuals, non-profits, education, and all development and
  testing.
- **Commercial license** for companies running it in production.
- **Enterprise services:** deployment, migration off Microsoft 365 or Google
  Workspace, custom connectors, support with SLAs, long-term-support releases.
- **Sponsorships and grants:** GitHub Sponsors, Open Collective, and public
  digital-sovereignty programs.

## Principles

1. Your data stays in your storage. FreeSaaS never needs its own copy.
2. Deployable by one developer with one config file.
3. Connectors are small and isolated, so the community can add more.
4. Upstream first: improve the editors for everyone, not just for us.
