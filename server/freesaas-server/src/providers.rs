//! Turns `freesaas.yaml` entries into provider instances. Each new provider
//! adds one match arm here.

use std::sync::Arc;

use freesaas_core::config::ProviderConfig;
use freesaas_core::{IdentityProvider, StorageProvider};

pub fn identity(cfg: &ProviderConfig) -> Result<Arc<dyn IdentityProvider>, String> {
    match cfg.kind.as_str() {
        "dev" => {
            if cfg.settings.get("insecure").and_then(|v| v.as_bool()) != Some(true) {
                return Err("identity type `dev` logs everyone in as one user; set `insecure: true` to confirm this is a local test".into());
            }
            Ok(Arc::new(freesaas_identity_dev::DevIdentity::new(
                cfg.str("user").unwrap_or("developer"),
            )))
        }
        // Planned: "oidc" (Entra ID, Google, Okta, Keycloak, Authentik), "saml", "ldap", "local".
        other => Err(format!("unknown identity provider type `{other}`")),
    }
}

pub fn storage(cfg: &ProviderConfig) -> Result<Arc<dyn StorageProvider>, String> {
    match cfg.kind.as_str() {
        "local" => {
            let root = cfg
                .str("root")
                .ok_or("storage type `local` needs a `root` folder")?;
            Ok(Arc::new(freesaas_storage_local::LocalStorage::new(
                cfg.id_or_kind(),
                root,
            )))
        }
        // Planned: "s3", "onedrive", "sharepoint", "gdrive", "webdav", "nextcloud".
        other => Err(format!("unknown storage provider type `{other}`")),
    }
}
