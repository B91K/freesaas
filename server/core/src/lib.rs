//! Provider interfaces for FreeSaaS.
//!
//! Every storage backend (local disk, S3, OneDrive, Google Drive, WebDAV, ...)
//! implements [`StorageProvider`], and every login backend (OIDC, SAML, LDAP,
//! ...) implements [`IdentityProvider`]. The server only talks to these traits,
//! so adding a backend never touches the editors or the hub.

pub mod config;
pub mod identity;
pub mod storage;

pub use identity::{IdentityProvider, User};
pub use storage::{Entry, EntryKind, StorageProvider};

/// Errors any provider can return.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("not supported by this provider: {0}")]
    Unsupported(&'static str),
    #[error("provider error: {0}")]
    Backend(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Error::NotFound(e.to_string()),
            _ => Error::Backend(e.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
