-- 创建本地 API 服务相关表

CREATE TABLE IF NOT EXISTS user_local_api_settings (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    port INTEGER NOT NULL DEFAULT 8080,
    remote_access BOOLEAN NOT NULL DEFAULT FALSE,
    cors_origins JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT fk_user_local_api_settings_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT chk_user_local_api_settings_port CHECK (port BETWEEN 1 AND 65535),
    CONSTRAINT chk_user_local_api_settings_cors_origins CHECK (jsonb_typeof(cors_origins) = 'array')
);

CREATE INDEX idx_user_local_api_settings_user_uuid
    ON user_local_api_settings(user_uuid);

CREATE TRIGGER update_user_local_api_settings_updated_at
    BEFORE UPDATE ON user_local_api_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS user_local_api_keys (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    key_prefix VARCHAR(32) NOT NULL,
    key_hash VARCHAR(128) NOT NULL,
    masked_key VARCHAR(64) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    requests_today INTEGER NOT NULL DEFAULT 0,
    daily_limit INTEGER NOT NULL DEFAULT 1000,
    last_reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT fk_user_local_api_keys_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT chk_user_local_api_keys_requests_today CHECK (requests_today >= 0),
    CONSTRAINT chk_user_local_api_keys_daily_limit CHECK (daily_limit >= 0)
);

CREATE INDEX idx_user_local_api_keys_user_uuid
    ON user_local_api_keys(user_uuid);

CREATE INDEX idx_user_local_api_keys_key_prefix
    ON user_local_api_keys(key_prefix);

CREATE INDEX idx_user_local_api_keys_key_hash
    ON user_local_api_keys(key_hash);

CREATE UNIQUE INDEX idx_user_local_api_keys_active_user
    ON user_local_api_keys(user_uuid)
    WHERE is_active = TRUE AND deleted_at IS NULL;

CREATE TRIGGER update_user_local_api_keys_updated_at
    BEFORE UPDATE ON user_local_api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS user_local_api_key_permissions (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    api_key_id INTEGER NOT NULL,
    permission_code VARCHAR(128) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT fk_user_local_api_key_permissions_key FOREIGN KEY (api_key_id) REFERENCES user_local_api_keys(id) ON DELETE CASCADE,
    CONSTRAINT uq_user_local_api_key_permissions UNIQUE (api_key_id, permission_code),
    CONSTRAINT chk_user_local_api_key_permissions_minute CHECK (rate_limit_per_minute >= 0),
    CONSTRAINT chk_user_local_api_key_permissions_hour CHECK (rate_limit_per_hour >= 0)
);

CREATE INDEX idx_user_local_api_key_permissions_key
    ON user_local_api_key_permissions(api_key_id);

CREATE INDEX idx_user_local_api_key_permissions_code
    ON user_local_api_key_permissions(permission_code);

CREATE TRIGGER update_user_local_api_key_permissions_updated_at
    BEFORE UPDATE ON user_local_api_key_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS user_local_api_request_counters (
    id SERIAL PRIMARY KEY,
    api_key_id INTEGER NOT NULL,
    permission_code VARCHAR(128) NOT NULL,
    window_type VARCHAR(16) NOT NULL,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user_local_api_request_counters_key FOREIGN KEY (api_key_id) REFERENCES user_local_api_keys(id) ON DELETE CASCADE,
    CONSTRAINT uq_user_local_api_request_counters UNIQUE (api_key_id, permission_code, window_type, window_start),
    CONSTRAINT chk_user_local_api_request_counters_window_type CHECK (window_type IN ('minute', 'hour', 'day')),
    CONSTRAINT chk_user_local_api_request_counters_request_count CHECK (request_count >= 0)
);

CREATE INDEX idx_user_local_api_request_counters_key
    ON user_local_api_request_counters(api_key_id);

CREATE INDEX idx_user_local_api_request_counters_lookup
    ON user_local_api_request_counters(api_key_id, permission_code, window_type, window_start);
