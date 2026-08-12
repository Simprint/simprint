-- 创建 user_extensions 表
-- 用户安装的扩展（用户级别）

CREATE TABLE IF NOT EXISTS user_extensions (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    extension_id VARCHAR(255) NOT NULL,
    installed_version VARCHAR(50) NOT NULL,
    -- 状态: installed, disabled
    status VARCHAR(50) NOT NULL DEFAULT 'installed',
    -- 时间
    installed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_user_extensions_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_user_extensions_ext FOREIGN KEY (extension_id) REFERENCES extensions(extension_id) ON DELETE CASCADE,
    -- 唯一约束
    CONSTRAINT uk_user_extensions UNIQUE (user_uuid, extension_id)
);

-- 创建索引
CREATE INDEX idx_user_extensions_user_uuid ON user_extensions(user_uuid);
CREATE INDEX idx_user_extensions_extension_id ON user_extensions(extension_id);

-- 创建更新时间触发器
CREATE TRIGGER update_user_extensions_updated_at BEFORE UPDATE ON user_extensions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

