//! Conversation domain: lifecycle (create, rename, archive, restore,
//! delete) and message pagination, plus the queries for the
//! conversations and messages tables.

mod service;
pub mod store;

pub use service::ConversationService;
