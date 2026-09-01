//! ConnectRPC bindings generated from `proto/`, plus the business-error
//! helpers shared by all domain crates.
//!
//! New `.proto` files must also be registered in this crate's `build.rs`;
//! the file list there is explicit and nothing scans the directory.

/// Generated message and service types, rooted at `lemma::v1`.
pub mod proto {
    connectrpc::include_generated!();
}

pub mod error;

pub use error::{app_error, app_error_with, error_reason};
pub use proto::lemma;
