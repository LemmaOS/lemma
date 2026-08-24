-- 会话内消息序号：确定性编码插入顺序，取代 created_at+id 排序
ALTER TABLE messages
ADD COLUMN seq BIGINT NOT NULL DEFAULT 0;
-- 存量回填：每会话按 (created_at, id) 排成 1..N
UPDATE messages m
SET seq = numbered.rn
FROM (
        SELECT id,
            row_number() OVER (
                PARTITION BY conversation_id
                ORDER BY created_at,
                    id
            ) AS rn
        FROM messages
    ) numbered
WHERE m.id = numbered.id;
CREATE INDEX ON messages (conversation_id, seq);
DROP INDEX IF EXISTS messages_conversation_id_created_at_id_idx;
