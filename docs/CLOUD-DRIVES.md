# Connecting OneDrive and Google Drive

The hub can open and save files in a user's own OneDrive or Google Drive
straight from the browser. There is no server and no secret: each user signs
in with their own account, and the hub only needs public **client ids**.
This works on GitHub Pages and on a self-hosted server alike.

## 1. Microsoft (OneDrive, SharePoint)

1. Go to the [Microsoft Entra admin center](https://entra.microsoft.com):
   **App registrations → New registration**.
2. **Supported account types:** "Accounts in any organizational directory and
   personal Microsoft accounts". For one company only, choose single tenant
   and set `tenant` below to your tenant id.
3. **Redirect URI:** platform **Single-page application**. Use the hub's exact
   address, for example `https://b91k.github.io/freesaas/`. It must be SPA,
   not "Web": SPA is what allows sign-in without a secret.
4. **API permissions:** Microsoft Graph, delegated: `User.Read`,
   `Files.ReadWrite`, `offline_access`.
5. Copy the **Application (client) id**.

## 2. Google (Google Drive)

1. In [Google Cloud Console](https://console.cloud.google.com), create a
   project and enable the **Google Drive API**.
2. **OAuth consent screen:** add the scope
   `https://www.googleapis.com/auth/drive.file`.
3. **Credentials → Create credentials → OAuth client id → Web application.**
   - Authorized JavaScript origins: `https://b91k.github.io`
   - Authorized redirect URIs: `https://b91k.github.io/freesaas/`
4. Copy the **client id**. The client secret is not used.

`drive.file` lets the hub see only the files it created or that the user
opened with it. Seeing a user's whole Drive needs the `drive` scope, which
Google treats as restricted: it requires verification and a security
assessment before public use.

## 3. Tell the hub

Put the ids in `config.json`. On GitHub Pages, set the repository variable
**FREESAAS_PUBLIC_CONFIG** (Settings → Secrets and variables → Actions →
**Variables**, not Secrets) to:

```json
{
  "mode": "static",
  "onedrive": { "clientId": "<application-client-id>", "tenant": "common" },
  "gdrive": { "clientId": "<google-client-id>.apps.googleusercontent.com" }
}
```

Then re-run the Pages workflow (Actions → Pages → Run workflow). Leave out
either entry to hide that drive.

## How it works

- **Microsoft:** authorization-code flow with PKCE, for single-page apps.
  Access tokens are refreshed automatically.
- **Google:** the browser token flow. Tokens last about an hour, then the
  user signs in again.
- Tokens are kept in `sessionStorage`: they last for that browser tab and
  are gone when it closes.
- Code: `web/cloud.js`.
