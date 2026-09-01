//! In-process registry of live generation streams.
//!
//! Each streaming assistant message gets a [`StreamHandle`] that
//! accumulates content, fans events out to any number of subscribers,
//! and accepts aborts. Handles live only in this process: a server
//! restart mid-stream is detected and finalized by the chat service.

use std::sync::{Arc, Mutex, MutexGuard};

use dashmap::DashMap;
use tokio::sync::{Notify, broadcast};
use uuid::Uuid;

use lemma_db::entity::TokenUsage;

/// An event broadcast to the subscribers of a live stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Delta(String),
    Done(Option<TokenUsage>),
    Aborted,
    Failed(String),
}

/// Lifecycle state of a stream. Terminal states are sticky.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStatus {
    Live,
    Done,
    Aborted,
    Failed,
}

/// Shared state of one streaming message.
pub struct StreamHandle {
    content: Mutex<String>,
    status: Mutex<StreamStatus>,
    tx: broadcast::Sender<StreamEvent>,
    abort: Notify,
}

impl StreamHandle {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            content: Mutex::new(String::new()),
            status: Mutex::new(StreamStatus::Live),
            tx,
            abort: Notify::new(),
        }
    }

    /// Appends generated text and broadcasts it.
    pub fn push_delta(&self, chunk: &str) {
        self.lock_content().push_str(chunk);
        let _ = self.tx.send(StreamEvent::Delta(chunk.to_owned()));
    }

    /// Marks the stream finished and broadcasts the final usage.
    pub fn finish(&self, usage: Option<TokenUsage>) {
        *self.lock_status() = StreamStatus::Done;
        let _ = self.tx.send(StreamEvent::Done(usage));
    }

    /// Marks the stream aborted by the client.
    pub fn mark_aborted(&self) {
        *self.lock_status() = StreamStatus::Aborted;
        let _ = self.tx.send(StreamEvent::Aborted);
    }

    /// Marks the stream failed with an upstream or internal error.
    pub fn fail(&self, message: &str) {
        *self.lock_status() = StreamStatus::Failed;
        let _ = self.tx.send(StreamEvent::Failed(message.to_owned()));
    }

    /// Requests abort. Returns false once the stream reached a terminal
    /// state, so a late abort cannot clobber a finished message.
    pub fn abort(&self) -> bool {
        if *self.lock_status() != StreamStatus::Live {
            return false;
        }
        self.abort.notify_one();
        true
    }

    /// Resolves when an abort is requested.
    pub async fn aborted(&self) {
        self.abort.notified().await;
    }

    /// Returns the content accumulated past `offset` (in chars) plus a
    /// live subscription. The content lock is held while subscribing, so
    /// no delta can fall between the replay and the live feed.
    pub fn snapshot_and_subscribe(
        &self,
        offset: usize,
    ) -> (String, broadcast::Receiver<StreamEvent>) {
        let content = self.lock_content();
        let rx = self.tx.subscribe();
        let replay: String = content.chars().skip(offset).collect();
        (replay, rx)
    }

    pub fn status(&self) -> StreamStatus {
        *self.lock_status()
    }

    pub fn content(&self) -> String {
        self.lock_content().clone()
    }

    // A poisoned lock still holds valid content here; recover it instead
    // of panicking the whole stream.
    fn lock_content(&self) -> MutexGuard<'_, String> {
        self.content.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn lock_status(&self) -> MutexGuard<'_, StreamStatus> {
        self.status.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Maps message ids to their live stream handles. Entries are removed
/// when the driving task ends.
#[derive(Clone, Default)]
pub struct StreamRegistry {
    inner: Arc<DashMap<Uuid, Arc<StreamHandle>>>,
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, message_id: Uuid) -> Arc<StreamHandle> {
        let handle = Arc::new(StreamHandle::new());
        self.inner.insert(message_id, Arc::clone(&handle));
        handle
    }

    pub fn get(&self, message_id: &Uuid) -> Option<Arc<StreamHandle>> {
        self.inner.get(message_id).map(|h| h.value().clone())
    }

    pub fn remove(&self, message_id: &Uuid) {
        self.inner.remove(message_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_replays_from_char_offset() {
        let h = StreamRegistry::new().register(Uuid::new_v4());
        h.push_delta("你好");
        h.push_delta("世界");
        let (replay, _rx) = h.snapshot_and_subscribe(2);
        assert_eq!(replay, "世界");
    }

    #[tokio::test]
    async fn subscriber_receives_live_deltas() {
        let h = StreamRegistry::new().register(Uuid::new_v4());
        let (_replay, mut rx) = h.snapshot_and_subscribe(0);
        h.push_delta("hi");
        match rx.recv().await {
            Ok(StreamEvent::Delta(s)) => assert_eq!(s, "hi"),
            _ => panic!("expected delta"),
        }
    }

    #[tokio::test]
    async fn abort_is_idempotent_and_sticky() {
        let h = StreamRegistry::new().register(Uuid::new_v4());
        assert!(h.abort());
        h.mark_aborted();
        assert!(!h.abort());
        h.aborted().await;
    }
}
