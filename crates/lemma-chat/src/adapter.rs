//! 供应商协议适配层：把各家 API 的流式响应统一成 AdapterEvent 流

use std::future::Future;
use std::pin::Pin;

use futures::{Stream, StreamExt, stream};
use serde::Deserialize;

use lemma_db::entity::TokenUsage;

pub type BoxEventStream = Pin<Box<dyn Stream<Item = Result<AdapterEvent, AdapterError>> + Send>>;

pub type BoxChatFuture = Pin<Box<dyn Future<Output = Result<BoxEventStream, AdapterError>> + Send>>;

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

// ---------- OpenAI Compatible ----------

pub struct OpenAiCompatible {
    client: reqwest::Client,
}

impl OpenAiCompatible {
    pub fn new() -> Self {
        // 流式响应不设整体超时
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

enum SseLine {
    Event(Result<AdapterEvent, AdapterError>),
    End,
    Skip,
}

fn parse_sse_line(line: &str) -> SseLine {
    let data = match line.strip_prefix("data:") {
        Some(d) => d.trim(),
        None => return SseLine::Skip,
    };
    if data == "[DONE]" {
        return SseLine::End;
    }
    let chunk: StreamChunk = match serde_json::from_str(data) {
        Ok(c) => c,
        Err(e) => {
            return SseLine::Event(Err(AdapterError::protocol(format!("bad chunk: {e}"))));
        }
    };
    // 末段 usage chunk 的 choices 为空，须先判 usage
    if let Some(usage) = chunk.usage {
        return SseLine::Event(Ok(AdapterEvent::Done(Some(usage.into()))));
    }
    let content = chunk
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.delta)
        .and_then(|d| d.content);
    match content {
        Some(text) if !text.is_empty() => SseLine::Event(Ok(AdapterEvent::Delta(text))),
        _ => SseLine::Skip,
    }
}

type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AdapterError>> + Send>>;

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
                .map_err(AdapterError::transport)?;
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.map_err(AdapterError::transport)?;
                return Err(AdapterError::http(status, text));
            }
            let bytes: ByteStream = Box::pin(
                resp.bytes_stream()
                    .map(|r| r.map(|b| b.to_vec()).map_err(AdapterError::transport)),
            );
            let s = stream::try_unfold((bytes, Vec::<u8>::new(), false), {
                |(mut bytes, mut buf, finished)| async move {
                    if finished {
                        return Ok(None);
                    }
                    loop {
                        // 只处理完整行；\n 不会出现在 UTF-8 多字节序列内，按字节切安全
                        if let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                            let raw: Vec<u8> = buf.drain(..=pos).collect();
                            let line = String::from_utf8_lossy(&raw);
                            match parse_sse_line(line.trim_end_matches('\r')) {
                                SseLine::Skip => continue,
                                SseLine::End => {
                                    return Ok(Some((
                                        AdapterEvent::Done(None),
                                        (bytes, buf, true),
                                    )));
                                }
                                SseLine::Event(Ok(e @ AdapterEvent::Delta(_))) => {
                                    return Ok(Some((e, (bytes, buf, false))));
                                }
                                SseLine::Event(Ok(e @ AdapterEvent::Done(_))) => {
                                    return Ok(Some((e, (bytes, buf, true))));
                                }
                                SseLine::Event(Err(e)) => return Err(e),
                            }
                        }
                        match bytes.next().await {
                            Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                            Some(Err(e)) => return Err(e),
                            // 上游断开但未发 [DONE]：以残余缓冲收尾
                            None if buf.is_empty() => return Ok(None),
                            None => {
                                let line = String::from_utf8_lossy(&buf).into_owned();
                                buf.clear();
                                match parse_sse_line(line.trim()) {
                                    SseLine::Event(Ok(e @ AdapterEvent::Delta(_))) => {
                                        return Ok(Some((e, (bytes, buf, true))));
                                    }
                                    _ => {
                                        return Ok(Some((
                                            AdapterEvent::Done(None),
                                            (bytes, buf, true),
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            });
            Ok(Box::pin(s) as BoxEventStream)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_delta() {
        match parse_sse_line(r#"data: {"choices":[{"delta":{"content":"你好"}}]}"#) {
            SseLine::Event(Ok(AdapterEvent::Delta(s))) => assert_eq!(s, "你好"),
            _ => panic!("expected delta"),
        }
    }

    #[test]
    fn skip_role_only_and_empty() {
        assert!(matches!(
            parse_sse_line(r#"data: {"choices":[{"delta":{"role":"assistant"}}]}"#),
            SseLine::Skip
        ));
        assert!(matches!(parse_sse_line(""), SseLine::Skip));
        assert!(matches!(parse_sse_line(": ping"), SseLine::Skip));
    }

    #[test]
    fn parse_usage_then_done() {
        match parse_sse_line(
            r#"data: {"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":3,"total_tokens":15}}"#,
        ) {
            SseLine::Event(Ok(AdapterEvent::Done(Some(u)))) => {
                assert_eq!(u.total, 15);
            }
            _ => panic!("expected done with usage"),
        }
        assert!(matches!(parse_sse_line("data: [DONE]"), SseLine::End));
    }

    #[test]
    fn bad_json_is_error() {
        assert!(matches!(
            parse_sse_line("data: {not json"),
            SseLine::Event(Err(_))
        ));
    }
}
