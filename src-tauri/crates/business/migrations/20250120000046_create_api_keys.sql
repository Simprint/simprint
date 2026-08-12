-- 创建 api_keys 表
-- API 密钥表

CREATE TABLE IF NOT EXISTS api_keys (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 密钥信息
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(20) NOT NULL,
    -- 权限: read, write, delete, admin
    permissions JSONB NOT NULL DEFAULT '["read"]',
    -- 限流
    rate_limit INT DEFAULT 1000,
    daily_limit INT DEFAULT 10000,
    -- IP 白名单（JSON 数组）
    ip_whitelist JSONB DEFAULT '[]',
    -- 过期时间
    expires_at TIMESTAMP WITH TIME ZONE,
    -- 使用统计
    usage_count BIGINT DEFAULT 0,
    daily_usage INT DEFAULT 0,
    last_used_at TIMESTAMP WITH TIME ZONE,
    -- 状态: active, revoked
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_api_keys_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_api_keys_user_uuid ON api_keys(user_uuid);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_key_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_status ON api_keys(status);

-- 创建更新时间触发器
CREATE TRIGGER update_api_keys_updated_at BEFORE UPDATE ON api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

