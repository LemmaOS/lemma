//! Sync domain: pull-based replication of conversations and messages,
//! plus a watch stream that hints at changes.

mod service;
pub mod store;

pub use service::SyncService;
