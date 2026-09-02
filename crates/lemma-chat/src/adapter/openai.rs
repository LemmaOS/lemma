//! Adapter for OpenAI-compatible chat completions APIs.

use serde::Deserialize;

use lemma_db::entity::TokenUsage;

use super::sse::{Parsed, SseParser, events_from_sse};
use super::{BoxChatFuture, ChatRequest, LlmAdapter, bytes_of};

/// Streams via `POST <base_url><api_path>/chat/completions` with a
/// bearer token.
pub struct OpenAiCompatible {
    client: reqwest::Client,
}

impl OpenAiCompatible {
    /// Creates the adapter.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for OpenAiCompatible {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

impl From<Usage> for TokenUsage {
    fn from(u: Usage) -> Self {
        Self {
            prompt: u.prompt_tokens,
            completion: u.completion_tokens,
            total: u.total_tokens,
        }
    }
}

// With stream_options.include_usage set, usage arrives in its own chunk
// ahead of [DONE]; the parser holds it until the terminal event.
struct Parser {
    usage: Option<Usage>,
}

impl SseParser for Parser {
    fn parse_line(&mut self, data: &str) -> Result<Parsed, super::AdapterError> {
        if data == "[DONE]" {
            return Ok(Parsed::Finish(self.usage.take().map(Into::into)));
        }
        let chunk: StreamChunk = serde_json::from_str(data)
            .map_err(|e| super::AdapterError::protocol(format!("bad chunk: {e}")))?;
        if let Some(usage) = chunk.usage {
            self.usage = Some(usage);
            return Ok(Parsed::Skip);
        }
        let content = chunk
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.delta)
            .and_then(|d| d.content);
        match content {
            Some(text) if !text.is_empty() => Ok(Parsed::Delta(text)),
            _ => Ok(Parsed::Skip),
        }
    }

    fn on_eof(&mut self) -> Option<TokenUsage> {
        self.usage.take().map(Into::into)
    }
}

impl LlmAdapter for OpenAiCompatible {
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture {
        let client = self.client.clone();
        Box::pin(async move {
            let path = if req.api_path.is_empty() {
                "/chat/completions"
            } else {
                &req.api_path
            };
            let url = format!("{}{}", req.base_url.trim_end_matches('/'), path);
            let body = serde_json::json!({
                "model": req.model,
                "messages": req.messages.iter().map(|m| serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })).collect::<Vec<_>>(),
                "stream": true,
                "stream_options": { "include_usage": true },
            });
            let resp = client
                .post(&url)
                .bearer_auth(&req.api_key)
                .json(&body)
                .send()
                .await
                .map_err(super::AdapterError::transport)?;
            Ok(events_from_sse(
                bytes_of(resp).await?,
                Parser { usage: None },
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> Parser {
        Parser { usage: None }
    }

    #[test]
    fn parse_delta() {
        match parser().parse_line(r#"{"choices":[{"delta":{"content":"你好"}}]}"#) {
            Ok(Parsed::Delta(s)) => assert_eq!(s, "你好"),
            _ => panic!("expected delta"),
        }
    }

    #[test]
    fn skip_role_only_and_empty() {
        assert!(matches!(
            parser().parse_line(r#"{"choices":[{"delta":{"role":"assistant"}}]}"#),
            Ok(Parsed::Skip)
        ));
        assert!(matches!(
            parser().parse_line(r#"{"choices":[]}"#),
            Ok(Parsed::Skip)
        ));
    }

    #[test]
    fn usage_then_done() {
        let mut p = parser();
        assert!(matches!(
            p.parse_line(
                r#"{"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":3,"total_tokens":15}}"#
            ),
            Ok(Parsed::Skip)
        ));
        match p.parse_line("[DONE]") {
            Ok(Parsed::Finish(Some(u))) => assert_eq!(u.total, 15),
            _ => panic!("expected finish with usage"),
        }
    }

    #[test]
    fn eof_flushes_pending_usage() {
        let mut p = parser();
        let _ = p.parse_line(
            r#"{"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":3,"total_tokens":15}}"#,
        );
        let u = p.on_eof();
        assert!(matches!(u, Some(TokenUsage { total: 15, .. })));
    }

    #[test]
    fn bad_json_is_error() {
        assert!(parser().parse_line("{not json").is_err());
    }

    #[test]
    fn done_without_usage() {
        assert!(matches!(
            parser().parse_line("[DONE]"),
            Ok(Parsed::Finish(None))
        ));
    }
}
