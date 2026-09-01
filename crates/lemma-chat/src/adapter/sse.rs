//! SSE line decoding shared by the adapters.

use std::pin::Pin;

use futures::{Stream, StreamExt, stream};

use lemma_db::entity::TokenUsage;

use super::{AdapterError, AdapterEvent, BoxEventStream, ByteStream};

/// Outcome of parsing one `data:` payload.
#[derive(Debug)]
pub enum Parsed {
    /// The payload carries no text for the client.
    Skip,
    /// A chunk of generated text.
    Delta(String),
    /// The terminal event, optionally with token usage.
    Finish(Option<TokenUsage>),
}

/// Parses a provider's SSE `data:` payloads into events.
pub trait SseParser: Send + 'static {
    fn parse_line(&mut self, data: &str) -> Result<Parsed, AdapterError>;
    /// Flushes pending state when the stream ends without a terminal
    /// event; used for token usage on APIs with no `[DONE]` sentinel.
    fn on_eof(&mut self) -> Option<TokenUsage> {
        None
    }
}

type ByteLines = Pin<Box<dyn Stream<Item = Result<String, AdapterError>> + Send>>;

fn lines(bytes: ByteStream) -> ByteLines {
    Box::pin(stream::try_unfold(
        (bytes, Vec::<u8>::new()),
        |(mut bytes, mut buf)| async move {
            loop {
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

/// Turns an upstream byte stream into adapter events. Only `data:` lines
/// are considered; `Done` is emitted exactly once, at the terminal event
/// or at EOF.
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
                        // The stream ended without a terminal event;
                        // synthesize Done from whatever the parser held.
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
