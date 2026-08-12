-- 创建 environment_configs 表
-- 环境完整配置表（存储 WindowConfig，与 environments 1:1）

CREATE TABLE IF NOT EXISTS environment_configs (
    id SERIAL PRIMARY KEY,
    environment_uuid UUID NOT NULL UNIQUE,
    -- WindowInfo
    window_info JSONB NOT NULL DEFAULT '{}',
    -- BasicSettings
    basic_settings JSONB NOT NULL DEFAULT '{}',
    -- AdvancedFingerprintSettings
    fingerprint_settings JSONB NOT NULL DEFAULT '{}',
    -- DeviceSettings
    device_settings JSONB NOT NULL DEFAULT '{}',
    -- PreferenceSettings
    preference_settings JSONB NOT NULL DEFAULT '{}',
    -- ProjectMetadata
    project_metadata JSONB NOT NULL DEFAULT '{}',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_configs_env FOREIGN KEY (environment_uuid) 
        REFERENCES environments(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_env_configs_env_uuid ON environment_configs(environment_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_environment_configs_updated_at BEFORE UPDATE ON environment_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

