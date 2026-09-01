//! Adapter for the Gemini generateContent API.

use serde::Deserialize;

use lemma_db::entity::TokenUsage;

use super::sse::{Parsed, SseParser, events_from_sse};
use super::{AdapterError, BoxChatFuture, ChatRequest, LlmAdapter, bytes_of};

/// Streams via `:streamGenerateContent?alt=sse` with an `x-goog-api-key`
/// header. The stream has no `[DONE]` sentinel; usage rides the last
/// chunk and is flushed at EOF.
pub struct GeminiGenerate {
    client: reqwest::Client,
}

impl GeminiGenerate {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for GeminiGenerate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage: Option<UsageMeta>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<Content>,
}

#[derive(Deserialize)]
struct Content {
    #[serde(default)]
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: Option<String>,
}

#[derive(Deserialize)]
struct UsageMeta {
    #[serde(rename = "promptTokenCount")]
    prompt: Option<i64>,
    #[serde(rename = "candidatesTokenCount")]
    completion: Option<i64>,
    #[serde(rename = "totalTokenCount")]
    total: Option<i64>,
}

impl From<UsageMeta> for TokenUsage {
    fn from(u: UsageMeta) -> Self {
        let prompt = u.prompt.unwrap_or(0);
        let completion = u.completion.unwrap_or(0);
        Self {
            prompt,
            completion,
            total: u.total.unwrap_or(prompt + completion),
        }
    }
}

struct Parser {
    usage: Option<UsageMeta>,
}

impl SseParser for Parser {
    fn parse_line(&mut self, data: &str) -> Result<Parsed, AdapterError> {
        let chunk: Chunk = serde_json::from_str(data)
            .map_err(|e| AdapterError::protocol(format!("bad chunk: {e}")))?;
        if let Some(u) = chunk.usage {
            self.usage = Some(u);
        }
        let text: String = chunk
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content)
            .map(|c| {
                c.parts
                    .into_iter()
                    .filter_map(|p| p.text)
                    .collect::<String>()
            })
            .unwrap_or_default();
        if text.is_empty() {
            Ok(Parsed::Skip)
        } else {
            Ok(Parsed::Delta(text))
        }
    }

    fn on_eof(&mut self) -> Option<TokenUsage> {
        self.usage.take().map(Into::into)
    }
}

impl LlmAdapter for GeminiGenerate {
    fn stream_chat(&self, req: ChatRequest) -> BoxChatFuture {
        let client = self.client.clone();
        Box::pin(async move {
            // A custom api_path embeds the model via a {model}
            // placeholder; the default path appends it directly.
            let path = if req.api_path.is_empty() {
                format!("/models/{}:streamGenerateContent?alt=sse", req.model)
            } else {
                req.api_path.replace("{model}", &req.model)
            };
            let url = format!("{}{}", req.base_url.trim_end_matches('/'), path);
            let body = serde_json::json!({
                // Gemini names the assistant role "model".
                "contents": req.messages.iter().map(|m| serde_json::json!({
                    "role": if m.role == "assistant" { "model" } else { "user" },
                    "parts": [{ "text": m.content }],
                })).collect::<Vec<_>>(),
            });
            let resp = client
                .post(&url)
                .header("x-goog-api-key", &req.api_key)
                .json(&body)
                .send()
                .await
                .map_err(AdapterError::transport)?;
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
    fn parse_text_part() {
        match parser().parse_line(
            r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"你好"}]}}]}"#,
        ) {
            Ok(Parsed::Delta(s)) => assert_eq!(s, "你好"),
            _ => panic!("expected delta"),
        }
    }

    #[test]
    fn multi_parts_concatenate() {
        match parser().parse_line(
            r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"你"},{"text":"好"}]}}]}"#,
        ) {
            Ok(Parsed::Delta(s)) => assert_eq!(s, "你好"),
            _ => panic!("expected concatenated delta"),
        }
    }

    #[test]
    fn skip_empty_candidates() {
        assert!(matches!(
            parser().parse_line(r#"{"candidates":[]}"#),
            Ok(Parsed::Skip)
        ));
    }

    #[test]
    fn usage_flushed_on_eof() {
        let mut p = parser();
        let _ = p.parse_line(
            r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"答"}]}}],"usageMetadata":{"promptTokenCount":9,"candidatesTokenCount":4,"totalTokenCount":13}}"#,
        );
        match p.on_eof() {
            Some(u) => {
                assert_eq!(u.prompt, 9);
                assert_eq!(u.completion, 4);
                assert_eq!(u.total, 13);
            }
            None => panic!("expected usage"),
        }
    }

    #[test]
    fn total_falls_back_to_sum() {
        let mut p = parser();
        let _ =
            p.parse_line(r#"{"usageMetadata":{"promptTokenCount":9,"candidatesTokenCount":4}}"#);
        assert!(matches!(p.on_eof(), Some(TokenUsage { total: 13, .. })));
    }

    #[test]
    fn bad_json_is_error() {
        assert!(parser().parse_line("{not json").is_err());
    }
}
