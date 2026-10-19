use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;

use crate::{Result, User};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Folder,
}

/// One item in a folder listing.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    /// Provider-relative path using `/` separators, e.g. `reports/q3.docx`.
    pub path: String,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    /// Seconds since the Unix epoch, when the backend knows it.
    pub modified: Option<u64>,
}

/// A place documents live. Paths are always relative to the provider root and
/// use `/` separators; an empty string is the root.
///
/// Calls carry the acting [`User`] so backends that act on the user's behalf
/// (OneDrive, Google Drive) can pick the right token, and so shared backends
/// (S3, local disk) can enforce per-user scoping.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Stable id from the config, e.g. `"local"` or `"company-s3"`.
    fn id(&self) -> &str;

    async fn list(&self, user: &User, folder: &str) -> Result<Vec<Entry>>;
    async fn read(&self, user: &User, path: &str) -> Result<Bytes>;
    async fn write(&self, user: &User, path: &str, data: Bytes) -> Result<Entry>;
    async fn delete(&self, user: &User, path: &str) -> Result<()>;
}
