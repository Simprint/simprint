-- Console Gateway API 密钥表
-- 用于存储 console-gateway 的 API 密钥

CREATE TABLE IF NOT EXISTS console_api_keys (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    key_id VARCHAR(64) NOT NULL UNIQUE,
    key_secret VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_by UUID,
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_console_api_keys_key_id ON console_api_keys(key_id);
CREATE INDEX IF NOT EXISTS idx_console_api_keys_is_active ON console_api_keys(is_active);
CREATE INDEX IF NOT EXISTS idx_console_api_keys_expires_at ON console_api_keys(expires_at);
CREATE INDEX IF NOT EXISTS idx_console_api_keys_deleted_at ON console_api_keys(deleted_at);

-- 注释
COMMENT ON TABLE console_api_keys IS 'Console Gateway API 密钥表';
COMMENT ON COLUMN console_api_keys.id IS '主键 ID';
COMMENT ON COLUMN console_api_keys.uuid IS '唯一标识';
COMMENT ON COLUMN console_api_keys.key_id IS 'API 密钥 ID (公开部分)';
COMMENT ON COLUMN console_api_keys.key_secret IS 'API 密钥密文 (私密部分)';
COMMENT ON COLUMN console_api_keys.name IS '密钥名称';
COMMENT ON COLUMN console_api_keys.description IS '密钥描述';
COMMENT ON COLUMN console_api_keys.is_active IS '是否激活';
COMMENT ON COLUMN console_api_keys.created_at IS '创建时间';
COMMENT ON COLUMN console_api_keys.updated_at IS '更新时间';
COMMENT ON COLUMN console_api_keys.expires_at IS '过期时间';
COMMENT ON COLUMN console_api_keys.last_used_at IS '最后使用时间';
COMMENT ON COLUMN console_api_keys.created_by IS '创建人 UUID';
COMMENT ON COLUMN console_api_keys.deleted_at IS '删除时间 (软删除)';

