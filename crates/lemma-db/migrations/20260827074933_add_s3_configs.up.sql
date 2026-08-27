-- 每用户单套（user_id UNIQUE）；密钥 AES-GCM 密封存 TEXT；不参与 sync_seq 流
-- migration_from：换后端时旧配置全量快照（含密封密钥），迁移完成置 NULL
CREATE TABLE s3_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    endpoint TEXT NOT NULL,
    region TEXT NOT NULL,
    bucket TEXT NOT NULL,
    access_key TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    migration_from JSONB,
    migrated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
