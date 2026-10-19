//! Treats every request as one fixed user, so the suite runs locally with no
//! login setup. The server refuses to start with it unless explicitly allowed.

use async_trait::async_trait;
use freesaas_core::{IdentityProvider, Result, User};

pub struct DevIdentity {
    user: User,
}

impl DevIdentity {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            user: User {
                id: format!("dev-{name}"),
                name,
                email: None,
                groups: vec!["dev".into()],
            },
        }
    }
}

#[async_trait]
impl IdentityProvider for DevIdentity {
    fn id(&self) -> &str {
        "dev"
    }

    async fn authenticate(&self, _credential: Option<&str>) -> Result<User> {
        Ok(self.user.clone())
    }
}
