//! 增量同步：sync_seq 定序的 Pull + 常驻 Watch 流

mod service;
pub mod store;

pub use service::SyncService;
