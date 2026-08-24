//! 供应商协议适配层：把各家 API 的流式响应统一成 AdapterEvent 流

mod anthropic;
mod gemini;
mod openai;
mod sse;

use std::future::Future;
use std::pin::Pin;

use futures::{Stream, StreamExt};

use lemma_db::entity::TokenUsage;
use lemma_proto::lemma::v1::ProviderKind;

pub use anthropic::AnthropicMessages;
pub use gemini::GeminiGenerate;
pub use openai::OpenAiCompatible;

pub type BoxEventStream = Pin<Box<dyn Stream<Item = Result<AdapterEvent, AdapterError>> + Send>>;

pub type BoxChatFuture = Pin<Box<dyn Future<Output = Result<BoxEventStream, AdapterError>> + Send>>;

pub(crate) type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AdapterError>> + Send>>;

/// 一次流式对话的输入
pub struct ChatRequest {
    pub kind: ProviderKind,
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

/// 按供应商类型分发到具体协议适配器；未知/未指定按 OpenAI 兼容兜底
pub struct DispatchAdapter {
    openai: OpenAiCompatible,
    anthropic: AnthropicMessages,
    gemini: GeminiGenerate,
}

impl DispatchAdapter {
    pub fn new() -> Self {
        Self {
            openai: OpenAiCompatible::new(),
            anthropic: AnthropicMessages::new(),
            gemini: GeminiGenerate::new(),
        }
    }

    fn select(&self, kind: ProviderKind) -> &dyn LlmAdapter {
        match kind {
            ProviderKind::PROVIDER_KIND_ANTHROPIC => &self.anthropic,
            ProviderKind::PROVIDER_KIND_GEMINI => &self.gemini,
            _ => &self.openai,
        }
    }
}

impl Default for DispatchAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmAdapter for DispatchAdapter {
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture {
        self.select(req.kind).stream_chat(req)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // 胖指针的 vtable 可能按转换点去重失败，只比数据指针
    fn same_data(a: &dyn LlmAdapter, b: &dyn LlmAdapter) -> bool {
        std::ptr::from_ref(a).cast::<()>() == std::ptr::from_ref(b).cast::<()>()
    }

    #[test]
    fn dispatch_by_kind() {
        let d = DispatchAdapter::new();
        assert!(same_data(d.select(ProviderKind::Anthropic), &d.anthropic));
        assert!(same_data(d.select(ProviderKind::Gemini), &d.gemini));
        assert!(same_data(d.select(ProviderKind::Openai), &d.openai));
        assert!(same_data(d.select(ProviderKind::Unspecified), &d.openai));
    }
}
