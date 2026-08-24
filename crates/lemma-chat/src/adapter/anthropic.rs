//! Anthropic Messages API：POST /messages，事件类型在 data JSON 的 type 字段里

use serde::Deserialize;

use lemma_db::entity::TokenUsage;

use super::sse::{Parsed, SseParser, events_from_sse};
use super::{AdapterError, BoxChatFuture, ChatRequest, LlmAdapter, bytes_of};

/// API 必填；流式场景下只是上限，不代表实际生成量
const MAX_TOKENS: u32 = 8192;
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicMessages {
    client: reqwest::Client,
}

impl AnthropicMessages {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for AnthropicMessages {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "type")]
    kind: String,
    delta: Option<Delta>,
    usage: Option<Usage>,
    message: Option<MessageStart>,
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct Delta {
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
}

#[derive(Deserialize)]
struct MessageStart {
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

struct Parser {
    input: Option<i64>,
    output: Option<i64>,
}

impl Parser {
    fn usage(&self) -> Option<TokenUsage> {
        if self.input.is_none() && self.output.is_none() {
            return None;
        }
        let prompt = self.input.unwrap_or(0);
        let completion = self.output.unwrap_or(0);
        Some(TokenUsage {
            prompt,
            completion,
            total: prompt + completion,
        })
    }

    fn merge(&mut self, u: &Usage) {
        if let Some(i) = u.input_tokens {
            self.input = Some(i);
        }
        if let Some(o) = u.output_tokens {
            self.output = Some(o);
        }
    }
}

impl SseParser for Parser {
    fn parse_line(&mut self, data: &str) -> Result<Parsed, AdapterError> {
        let event: Event = serde_json::from_str(data)
            .map_err(|e| AdapterError::protocol(format!("bad event: {e}")))?;
        match event.kind.as_str() {
            "content_block_delta" => match event.delta.and_then(|d| d.text) {
                Some(text) if !text.is_empty() => Ok(Parsed::Delta(text)),
                _ => Ok(Parsed::Skip),
            },
            "message_start" => {
                if let Some(u) = event.message.and_then(|m| m.usage) {
                    self.merge(&u);
                }
                Ok(Parsed::Skip)
            }
            "message_delta" => {
                if let Some(u) = event.usage {
                    self.merge(&u);
                }
                Ok(Parsed::Skip)
            }
            "message_stop" => Ok(Parsed::Finish(self.usage())),
            "error" => Err(AdapterError::protocol(format!(
                "anthropic error: {}",
                event.error.map(|e| e.message).unwrap_or_default()
            ))),
            // ping / content_block_start / content_block_stop 等
            _ => Ok(Parsed::Skip),
        }
    }

    fn on_eof(&mut self) -> Option<TokenUsage> {
        self.usage()
    }
}

impl LlmAdapter for AnthropicMessages {
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture {
        let client = self.client.clone();
        Box::pin(async move {
            let path = if req.api_path.is_empty() {
                "/messages"
            } else {
                &req.api_path
            };
            let url = format!("{}{}", req.base_url.trim_end_matches('/'), path);
            let body = serde_json::json!({
                "model": req.model,
                "max_tokens": MAX_TOKENS,
                "messages": req.messages.iter().map(|m| serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })).collect::<Vec<_>>(),
                "stream": true,
            });
            let resp = client
                .post(&url)
                .header("x-api-key", &req.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .json(&body)
                .send()
                .await
                .map_err(AdapterError::transport)?;
            Ok(events_from_sse(
                bytes_of(resp).await?,
                Parser {
                    input: None,
                    output: None,
                },
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> Parser {
        Parser {
            input: None,
            output: None,
        }
    }

    #[test]
    fn parse_text_delta() {
        match parser().parse_line(
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"你好"}}"#,
        ) {
            Ok(Parsed::Delta(s)) => assert_eq!(s, "你好"),
            _ => panic!("expected delta"),
        }
    }

    #[test]
    fn skip_control_events() {
        let mut p = parser();
        for line in [
            r#"{"type":"ping"}"#,
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            r#"{"type":"content_block_stop","index":0}"#,
        ] {
            assert!(matches!(p.parse_line(line), Ok(Parsed::Skip)));
        }
    }

    #[test]
    fn usage_accumulates_until_message_stop() {
        let mut p = parser();
        let _ = p.parse_line(
            r#"{"type":"message_start","message":{"usage":{"input_tokens":25,"output_tokens":1}}}"#,
        );
        let _ = p.parse_line(
            r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}"#,
        );
        match p.parse_line(r#"{"type":"message_stop"}"#) {
            Ok(Parsed::Finish(Some(u))) => {
                assert_eq!(u.prompt, 25);
                assert_eq!(u.completion, 15);
                assert_eq!(u.total, 40);
            }
            _ => panic!("expected finish with usage"),
        }
    }

    #[test]
    fn error_event_is_protocol_error() {
        match parser().parse_line(
            r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
        ) {
            Err(e) => assert!(e.message.contains("Overloaded")),
            Ok(parsed) => panic!("expected error, got {parsed:?}"),
        }
    }

    #[test]
    fn eof_flushes_partial_usage() {
        let mut p = parser();
        let _ = p.parse_line(
            r#"{"type":"message_start","message":{"usage":{"input_tokens":10,"output_tokens":1}}}"#,
        );
        let u = p.on_eof();
        assert!(matches!(u, Some(TokenUsage { prompt: 10, .. })));
    }

    #[test]
    fn bad_json_is_error() {
        assert!(parser().parse_line("{not json").is_err());
    }
}
