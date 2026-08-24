//! 供应商协议适配层：把各家 API 的流式响应统一成 AdapterEvent 流

mod anthropic;
mod gemini;
mod openai;
mod sse;

use std::future::Future;
use std::pin::Pin;

use futures::{Stream, StreamExt};

use lemma_db::entity::TokenUsage;

pub use anthropic::AnthropicMessages;
pub use gemini::GeminiGenerate;
pub use openai::OpenAiCompatible;

pub type BoxEventStream = Pin<Box<dyn Stream<Item = Result<AdapterEvent, AdapterError>> + Send>>;

pub type BoxChatFuture = Pin<Box<dyn Future<Output = Result<BoxEventStream, AdapterError>> + Send>>;

pub(crate) type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AdapterError>> + Send>>;

/// 一次流式对话的输入
pub struct ChatRequest {
    pub base_url: String,
    pub api_path: String,
    pub api_key: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 适配层产出的统一事件
#[derive(Debug)]
pub enum AdapterEvent {
    Delta(String),
    /// usage 为 None = 上游不支持或未返回 token 统计
    Done(Option<TokenUsage>),
}

#[derive(Debug)]
pub struct AdapterError {
    pub message: String,
}

impl AdapterError {
    fn transport(e: reqwest::Error) -> Self {
        Self {
            message: format!("transport: {e}"),
        }
    }
    fn http(status: reqwest::StatusCode, body: String) -> Self {
        Self {
            message: format!("upstream {status}: {body}"),
        }
    }
    fn protocol(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

pub trait LlmAdapter: Send + Sync {
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture;
}

/// 非 2xx 收全量 body 报错（不进入流式）；2xx 转字节流
pub(crate) async fn bytes_of(resp: reqwest::Response) -> Result<ByteStream, AdapterError> {
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.map_err(AdapterError::transport)?;
        return Err(AdapterError::http(status, text));
    }
    Ok(Box::pin(resp.bytes_stream().map(|r| {
        r.map(|b| b.to_vec()).map_err(AdapterError::transport)
    })))
}
