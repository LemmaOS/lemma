pub mod proto {
    connectrpc::include_generated!();
}

pub mod error;

pub use error::{app_error, app_error_with, error_reason};
pub use proto::lemma;
