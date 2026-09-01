//! Archive domain: per-user S3 storage configuration, the archive-object
//! envelope format, and storage migration between backends.

mod envelope;
mod memory;
mod s3;
mod service;
mod source;
pub mod store;

pub use envelope::{
    ArchiveEnvelope, ArchivedMessage, deserialize_envelope, envelope_from_messages,
    messages_from_envelope, serialize_envelope,
};
pub use memory::MemoryArchiveStore;
pub use s3::{S3ArchiveStore, S3Config};
pub use service::{StorageService, copy_archive_objects};
pub use source::{ArchiveSource, DbArchiveSource};

/// Any archive-layer failure, reduced to a message. These are internal
/// operational errors, so they carry no business error code.
#[derive(Debug)]
pub struct ArchiveError(pub String);

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "archive: {}", self.0)
    }
}

impl std::error::Error for ArchiveError {}

/// Object storage for archived conversations.
pub trait ArchiveStore: Send + Sync + 'static {
    /// Uploads an object, overwriting any existing one at `key`.
    fn put(
        &self,
        key: &str,
        content: &[u8],
    ) -> impl Future<Output = Result<(), ArchiveError>> + Send;
    /// Downloads an object, or returns `None` when the key does not
    /// exist.
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Vec<u8>>, ArchiveError>> + Send;
    /// Deletes an object. Deleting a missing key is not an error.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), ArchiveError>> + Send;
}

/// Object key for a conversation's archive: `archives/<id>.json`.
pub fn object_key(conversation_id: uuid::Uuid) -> String {
    format!("archives/{conversation_id}.json")
}
