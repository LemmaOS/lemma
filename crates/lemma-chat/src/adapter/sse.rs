//! 共享的 SSE 切分与事件流装配：各家适配器只需实现 SseParser

use std::pin::Pin;

use futures::{Stream, StreamExt, stream};

use lemma_db::entity::TokenUsage;

use super::{AdapterError, AdapterEvent, BoxEventStream, ByteStream};

/// 解析一行 SSE data 载荷的结果
#[derive(Debug)]
pub enum Parsed {
    Skip,
    Delta(String),
    /// 上游的显式结束信号（如 [DONE] / message_stop）
    Finish(Option<TokenUsage>),
}

/// 各家协议的 SSE 解析器：有状态（跨行累积 usage 等）
pub trait SseParser: Send + 'static {
    fn parse_line(&mut self, data: &str) -> Result<Parsed, AdapterError>;
    /// EOF 兜底：上游无显式结束标记时，补发 Done 用的 usage
    fn on_eof(&mut self) -> Option<TokenUsage> {
        None
    }
}

/// 字节流 → 行流：只产出完整行，EOF 时冲刷残余缓冲
type ByteLines = Pin<Box<dyn Stream<Item = Result<String, AdapterError>> + Send>>;

fn lines(bytes: ByteStream) -> ByteLines {
    Box::pin(stream::try_unfold(
        (bytes, Vec::<u8>::new()),
        |(mut bytes, mut buf)| async move {
            loop {
                // \n 不会出现在 UTF-8 多字节序列内，按字节切安全
                if let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                    let raw: Vec<u8> = buf.drain(..=pos).collect();
                    let line = String::from_utf8_lossy(&raw)
                        .trim_end_matches('\r')
                        .to_string();
                    return Ok(Some((line, (bytes, buf))));
                }
                match bytes.next().await {
                    Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                    Some(Err(e)) => return Err(e),
                    None if buf.is_empty() => return Ok(None),
                    None => {
                        let line = String::from_utf8_lossy(&buf).trim().to_string();
                        buf.clear();
                        return Ok(Some((line, (bytes, buf))));
                    }
                }
            }
        },
    ))
}

/// SSE 字节流 → 统一事件流；EOF 且无显式结束时补 Done(on_eof())
pub fn events_from_sse(bytes: ByteStream, parser: impl SseParser) -> BoxEventStream {
    let s = stream::try_unfold(
        (lines(bytes), parser, false),
        |(mut lines, mut parser, finished)| async move {
            if finished {
                return Ok(None);
            }
            loop {
                match lines.next().await {
                    Some(Ok(line)) => {
                        // 只看 data: 载荷；event:/注释/空行一律跳过
                        let data = match line.strip_prefix("data:") {
                            Some(d) => d.trim(),
                            None => continue,
                        };
                        match parser.parse_line(data) {
                            Ok(Parsed::Skip) => continue,
                            Ok(Parsed::Delta(text)) => {
                                return Ok(Some((
                                    AdapterEvent::Delta(text),
                                    (lines, parser, false),
                                )));
                            }
                            Ok(Parsed::Finish(usage)) => {
                                return Ok(Some((
                                    AdapterEvent::Done(usage),
                                    (lines, parser, true),
                                )));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    Some(Err(e)) => return Err(e),
                    None => {
                        return Ok(Some((
                            AdapterEvent::Done(parser.on_eof()),
                            (lines, parser, true),
                        )));
                    }
                }
            }
        },
    );
    Box::pin(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoParser;

    impl SseParser for EchoParser {
        fn parse_line(&mut self, data: &str) -> Result<Parsed, AdapterError> {
            match data {
                "[DONE]" => Ok(Parsed::Finish(None)),
                "" => Ok(Parsed::Skip),
                d => Ok(Parsed::Delta(d.to_string())),
            }
        }
    }

    #[tokio::test]
    async fn splits_lines_across_chunks() {
        let chunks: Vec<Result<Vec<u8>, AdapterError>> = vec![
            Ok(b"data: hello\nda".to_vec()),
            Ok(b"ta: world\ndata: [DONE]\n".to_vec()),
        ];
        let bytes: ByteStream = Box::pin(stream::iter(chunks));
        let events: Vec<_> = events_from_sse(bytes, EchoParser).collect().await;
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], Ok(AdapterEvent::Delta(d)) if d == "hello"));
        assert!(matches!(&events[1], Ok(AdapterEvent::Delta(d)) if d == "world"));
        assert!(matches!(&events[2], Ok(AdapterEvent::Done(None))));
    }

    #[tokio::test]
    async fn eof_residual_line_is_flushed() {
        let chunks: Vec<Result<Vec<u8>, AdapterError>> = vec![Ok(b"data: tail".to_vec())];
        let bytes: ByteStream = Box::pin(stream::iter(chunks));
        let events: Vec<_> = events_from_sse(bytes, EchoParser).collect().await;
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], Ok(AdapterEvent::Delta(d)) if d == "tail"));
        assert!(matches!(&events[1], Ok(AdapterEvent::Done(None))));
    }
}
