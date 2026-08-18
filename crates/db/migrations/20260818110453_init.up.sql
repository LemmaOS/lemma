-- 全局单调递增序列：所有变更的绝对时序依据
CREATE SEQUENCE sync_seq;
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'normal',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- 首个注册用户为 owner，防并发竞态
CREATE UNIQUE INDEX users_owner_unique ON users (role)
WHERE role = 'owner';
-- 轮换链：replaced_by 指向新 token，revoked_at 记吊销
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT UNIQUE NOT NULL,
    label TEXT,
    replaced_by UUID REFERENCES refresh_tokens(id),
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX ON refresh_tokens (user_id);
-- api_key 加密存储；models 为 JSONB 数组；不参与 sync_seq 流
CREATE TABLE providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    models JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON providers (user_id);
-- status: active | archived；title 空串，默认名由客户端按 locale 渲染
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active',
    archived_at TIMESTAMPTZ,
    archive_key TEXT,
    message_count INTEGER,
    sync_seq BIGINT NOT NULL DEFAULT nextval('sync_seq'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON conversations (user_id, sync_seq);
-- role: user | assistant | system；status: streaming | done | aborted | error
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    provider_id UUID REFERENCES providers(id),
    model TEXT,
    status TEXT NOT NULL DEFAULT 'done',
    token_usage JSONB,
    sync_seq BIGINT NOT NULL DEFAULT nextval('sync_seq'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- 三列索引支撑 (created_at, id) keyset 分页
CREATE INDEX ON messages (conversation_id, created_at, id);
CREATE INDEX ON messages (sync_seq);
