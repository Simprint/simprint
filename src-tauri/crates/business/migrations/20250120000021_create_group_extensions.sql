-- 创建 group_extensions 表
-- 分组安装的扩展（分组级别，该分组下所有环境可用）

CREATE TABLE IF NOT EXISTS group_extensions (
    id SERIAL PRIMARY KEY,
    group_uuid UUID NOT NULL,
    extension_id VARCHAR(255) NOT NULL,
    installed_version VARCHAR(50) NOT NULL,
    -- 安装者
    installed_by UUID NOT NULL,
    -- 状态: installed, disabled
    status VARCHAR(50) NOT NULL DEFAULT 'installed',
    -- 时间
    installed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_group_extensions_group FOREIGN KEY (group_uuid) REFERENCES groups(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_group_extensions_ext FOREIGN KEY (extension_id) REFERENCES extensions(extension_id) ON DELETE CASCADE,
    CONSTRAINT fk_group_extensions_user FOREIGN KEY (installed_by) REFERENCES users(uuid),
    -- 唯一约束
    CONSTRAINT uk_group_extensions UNIQUE (group_uuid, extension_id)
);

-- 创建索引
CREATE INDEX idx_group_extensions_group_uuid ON group_extensions(group_uuid);
CREATE INDEX idx_group_extensions_extension_id ON group_extensions(extension_id);

-- 创建更新时间触发器
CREATE TRIGGER update_group_extensions_updated_at BEFORE UPDATE ON group_extensions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

