use async_trait::async_trait;
use serde::Serialize;

use crate::Result;

/// A signed-in person, as reported by the identity provider.
#[derive(Debug, Clone, Serialize)]
pub struct User {
    /// Provider-unique, stable id (the OIDC `sub`, an LDAP DN, ...).
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
}

/// Who is using the suite. Implementations: OIDC (Entra ID, Google, Okta,
/// Keycloak, Authentik), SAML, LDAP, local accounts, and `dev` for testing.
#[async_trait]
pub trait IdentityProvider: Send + Sync {
    fn id(&self) -> &str;

    /// Resolve the bearer credential from a request (session cookie or token)
    /// to a user. Returns [`crate::Error::Unauthorized`] when it is missing or invalid.
    async fn authenticate(&self, credential: Option<&str>) -> Result<User>;
}
