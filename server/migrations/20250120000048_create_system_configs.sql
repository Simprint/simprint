-- 创建 system_configs 表
-- 系统配置表（全局配置）

CREATE TABLE IF NOT EXISTS system_configs (
    id SERIAL PRIMARY KEY,
    -- 配置键
    config_key VARCHAR(255) NOT NULL UNIQUE,
    -- 配置值（JSON）
    config_value JSONB NOT NULL DEFAULT '{}',
    -- 描述
    description TEXT,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_system_configs_key ON system_configs(config_key);

-- 创建更新时间触发器
CREATE TRIGGER update_system_configs_updated_at BEFORE UPDATE ON system_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

