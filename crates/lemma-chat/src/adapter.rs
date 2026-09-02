//! Per-kind LLM adapters and the SSE plumbing they share.

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

/// Stream of generation events produced by an adapter.
pub type BoxEventStream = Pin<Box<dyn Stream<Item = Result<AdapterEvent, AdapterError>> + Send>>;

/// Future that establishes the upstream connection and yields the event
/// stream.
pub type BoxChatFuture = Pin<Box<dyn Future<Output = Result<BoxEventStream, AdapterError>> + Send>>;

pub(crate) type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AdapterError>> + Send>>;

/// One streaming chat call against a provider.
#[allow(missing_docs)]
pub struct ChatRequest {
    pub kind: ProviderKind,
    pub base_url: String,
    pub api_path: String,
    /// Plaintext key, opened from its sealed form just before the call.
    pub api_key: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[allow(missing_docs)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// One generation event from the upstream stream.
#[derive(Debug)]
pub enum AdapterEvent {
    /// A chunk of generated text.
    Delta(String),
    /// Generation finished, optionally with token usage.
    Done(Option<TokenUsage>),
}

/// Any adapter failure, reduced to a message. These surface to clients
/// as in-band error events, not coded business errors.
#[derive(Debug)]
#[allow(missing_docs)]
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

/// A provider-specific streaming chat implementation.
pub trait LlmAdapter: Send + Sync {
    /// Starts a streaming chat call.
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture;
}

/// Routes chat requests to the adapter for the provider kind.
pub struct DispatchAdapter {
    openai: OpenAiCompatible,
    anthropic: AnthropicMessages,
    gemini: GeminiGenerate,
}

impl DispatchAdapter {
    /// Creates a dispatcher with one adapter per known provider kind.
    pub fn new() -> Self {
        Self {
            openai: OpenAiCompatible::new(),
            anthropic: AnthropicMessages::new(),
            gemini: GeminiGenerate::new(),
        }
    }

    // Unspecified and unrecognized kinds fall through to the
    // OpenAI-compatible adapter, the most common API shape.
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

/// Converts a response into a byte stream, turning non-2xx statuses into
/// an error carrying the response body.
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
