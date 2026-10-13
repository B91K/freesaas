//! Stores documents in a folder on the server, one subfolder per user.
//! This is the reference [`StorageProvider`]: copy it when writing a new one.

use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use bytes::Bytes;
use freesaas_core::{Entry, EntryKind, Error, Result, StorageProvider, User};

pub struct LocalStorage {
    id: String,
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(id: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            root: root.into(),
        }
    }

    /// Map a provider path into this user's folder, rejecting anything that
    /// could escape it (`..`, absolute paths, drive prefixes).
    fn resolve(&self, user: &User, path: &str) -> Result<PathBuf> {
        let mut out = self.root.join(user_dir(&user.id));
        for c in Path::new(path).components() {
            match c {
                Component::Normal(part) => out.push(part),
                Component::CurDir => {}
                _ => return Err(Error::InvalidPath(path.to_string())),
            }
        }
        Ok(out)
    }

    async fn entry(&self, rel: &str, full: &Path) -> Result<Entry> {
        let meta = tokio::fs::metadata(full).await?;
        Ok(Entry {
            path: rel.to_string(),
            name: rel.rsplit('/').next().unwrap_or(rel).to_string(),
            kind: if meta.is_dir() {
                EntryKind::Folder
            } else {
                EntryKind::File
            },
            size: if meta.is_dir() { 0 } else { meta.len() },
            modified: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
        })
    }
}

/// User ids come from the identity provider and may contain `/` or `:`;
/// keep only filename-safe characters.
fn user_dir(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn join(folder: &str, name: &str) -> String {
    let folder = folder.trim_matches('/');
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}/{name}")
    }
}

#[async_trait]
impl StorageProvider for LocalStorage {
    fn id(&self) -> &str {
        &self.id
    }

    async fn list(&self, user: &User, folder: &str) -> Result<Vec<Entry>> {
        let dir = self.resolve(user, folder)?;
        if folder.trim_matches('/').is_empty() {
            tokio::fs::create_dir_all(&dir).await?;
        }
        let mut entries = Vec::new();
        let mut rd = tokio::fs::read_dir(&dir).await?;
        while let Some(item) = rd.next_entry().await? {
            let name = item.file_name().to_string_lossy().into_owned();
            entries.push(self.entry(&join(folder, &name), &item.path()).await?);
        }
        entries.sort_by(|a, b| {
            (a.kind != EntryKind::Folder, &a.name).cmp(&(b.kind != EntryKind::Folder, &b.name))
        });
        Ok(entries)
    }

    async fn read(&self, user: &User, path: &str) -> Result<Bytes> {
        Ok(tokio::fs::read(self.resolve(user, path)?).await?.into())
    }

    async fn write(&self, user: &User, path: &str, data: Bytes) -> Result<Entry> {
        let full = self.resolve(user, path)?;
        if let Some(parent) = full.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full, &data).await?;
        self.entry(path.trim_matches('/'), &full).await
    }

    async fn delete(&self, user: &User, path: &str) -> Result<()> {
        let full = self.resolve(user, path)?;
        if tokio::fs::metadata(&full).await?.is_dir() {
            tokio::fs::remove_dir_all(full).await?;
        } else {
            tokio::fs::remove_file(full).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str) -> User {
        User {
            id: id.into(),
            name: id.into(),
            email: None,
            groups: vec![],
        }
    }

    #[tokio::test]
    async fn write_list_read_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let store = LocalStorage::new("local", tmp.path());
        let alice = user("alice");

        store
            .write(&alice, "reports/q3.docx", Bytes::from_static(b"hello"))
            .await
            .unwrap();
        let root = store.list(&alice, "").await.unwrap();
        assert_eq!(root.len(), 1);
        assert_eq!(root[0].kind, EntryKind::Folder);

        let files = store.list(&alice, "reports").await.unwrap();
        assert_eq!(files[0].path, "reports/q3.docx");
        assert_eq!(files[0].size, 5);
        assert_eq!(
            store.read(&alice, "reports/q3.docx").await.unwrap(),
            "hello"
        );

        store.delete(&alice, "reports").await.unwrap();
        assert!(store.list(&alice, "").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn users_are_isolated_and_paths_cannot_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let store = LocalStorage::new("local", tmp.path());
        store
            .write(&user("alice"), "secret.txt", Bytes::from_static(b"x"))
            .await
            .unwrap();

        assert!(store.list(&user("bob"), "").await.unwrap().is_empty());
        assert!(matches!(
            store.read(&user("bob"), "../alice/secret.txt").await,
            Err(Error::InvalidPath(_))
        ));
        assert!(matches!(
            store.read(&user("bob"), "/etc/passwd").await,
            Err(Error::InvalidPath(_))
        ));
        // An id crafted to look like a path stays inside its own folder.
        assert!(store.list(&user("../alice"), "").await.unwrap().is_empty());
    }
}
