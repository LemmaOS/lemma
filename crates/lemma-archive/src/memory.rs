//! In-memory [`ArchiveStore`] for tests.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::{ArchiveError, ArchiveStore};

/// In-memory object store used as the archive backend in tests.
#[derive(Default)]
pub struct MemoryArchiveStore {
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl MemoryArchiveStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ArchiveStore for MemoryArchiveStore {
    async fn put(&self, key: &str, content: &[u8]) -> Result<(), ArchiveError> {
        self.objects
            .lock()
            .map_err(|e| ArchiveError(format!("lock: {e}")))?
            .insert(key.to_owned(), content.to_vec());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ArchiveError> {
        Ok(self
            .objects
            .lock()
            .map_err(|e| ArchiveError(format!("lock: {e}")))?
            .get(key)
            .cloned())
    }

    async fn delete(&self, key: &str) -> Result<(), ArchiveError> {
        self.objects
            .lock()
            .map_err(|e| ArchiveError(format!("lock: {e}")))?
            .remove(key);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_get_delete_roundtrip() {
        let store = MemoryArchiveStore::new();
        store.put("k", b"v").await.unwrap();
        assert_eq!(store.get("k").await.unwrap(), Some(b"v".to_vec()));
        store.delete("k").await.unwrap();
        store.delete("k").await.unwrap();
        assert_eq!(store.get("k").await.unwrap(), None);
    }
}
