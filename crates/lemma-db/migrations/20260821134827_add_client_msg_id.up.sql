-- 客户端生成 ID：重发去重与乐观渲染关联；NULL = 无客户端 ID
ALTER TABLE messages
ADD COLUMN client_msg_id TEXT;
-- 同一会话内客户端 ID 唯一，支撑 SendMessage 幂等
CREATE UNIQUE INDEX messages_client_msg_id_unique ON messages (conversation_id, client_msg_id)
WHERE client_msg_id IS NOT NULL;
