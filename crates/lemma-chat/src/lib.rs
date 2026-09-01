//! Chat domain: streaming message generation through per-kind LLM
//! adapters, with an in-process stream registry for live fan-out, abort,
//! and resume.

pub mod adapter;
pub mod registry;
mod service;
pub mod store;

pub use service::ChatService;
