-- 创建 environment_extensions 表
-- 环境安装的扩展（环境级别）

CREATE TABLE IF NOT EXISTS environment_extensions (
    id SERIAL PRIMARY KEY,
    environment_uuid UUID NOT NULL,
    extension_id VARCHAR(255) NOT NULL,
    installed_version VARCHAR(50) NOT NULL,
    -- 状态: installed, disabled
    status VARCHAR(50) NOT NULL DEFAULT 'installed',
    -- 时间
    installed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_extensions_env FOREIGN KEY (environment_uuid) REFERENCES environments(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_env_extensions_ext FOREIGN KEY (extension_id) REFERENCES extensions(extension_id) ON DELETE CASCADE,
    -- 唯一约束
    CONSTRAINT uk_env_extensions UNIQUE (environment_uuid, extension_id)
);

-- 创建索引
CREATE INDEX idx_env_extensions_env_uuid ON environment_extensions(environment_uuid);
CREATE INDEX idx_env_extensions_extension_id ON environment_extensions(extension_id);

-- 创建更新时间触发器
CREATE TRIGGER update_environment_extensions_updated_at BEFORE UPDATE ON environment_extensions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

