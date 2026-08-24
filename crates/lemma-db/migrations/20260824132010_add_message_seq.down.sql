DROP INDEX IF EXISTS messages_conversation_id_seq_idx;
ALTER TABLE messages DROP COLUMN IF EXISTS seq;
CREATE INDEX ON messages (conversation_id, created_at, id);
