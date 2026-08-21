//! 对话编排：发消息（流式）、中断、断线续传

pub mod adapter;
pub mod registry;
mod service;
pub mod store;

pub use service::ChatService;
