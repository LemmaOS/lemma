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

#[derive(Debug)]
pub struct ArchiveError(pub String);

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "archive: {}", self.0)
    }
}

impl std::error::Error for ArchiveError {}

pub trait ArchiveStore: Send + Sync + 'static {
    fn put(
        &self,
        key: &str,
        content: &[u8],
    ) -> impl Future<Output = Result<(), ArchiveError>> + Send;
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Vec<u8>>, ArchiveError>> + Send;
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), ArchiveError>> + Send;
}

pub fn object_key(conversation_id: uuid::Uuid) -> String {
    format!("archives/{conversation_id}.json")
}
