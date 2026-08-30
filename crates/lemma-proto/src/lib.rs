// 生成代码挂载点：类型与服务定义在编译期从 proto/ 生成
pub mod proto {
    connectrpc::include_generated!();
}

pub mod error;

pub use error::{app_error, app_error_with, error_reason};
pub use proto::lemma;
