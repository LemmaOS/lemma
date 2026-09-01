//! Provider domain: CRUD for user-configured LLM providers, live model
//! list fetching, and the queries for the providers table.

mod models;
pub mod providers;
mod service;

pub use models::fetch_models;
pub use service::{ProviderService, kind_to_proto};
