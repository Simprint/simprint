-- 创建 team_extensions 表
-- 团队安装的扩展（团队级别，所有团队成员可用）

CREATE TABLE IF NOT EXISTS team_extensions (
    id SERIAL PRIMARY KEY,
    team_uuid UUID NOT NULL,
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
    CONSTRAINT fk_team_extensions_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_team_extensions_ext FOREIGN KEY (extension_id) REFERENCES extensions(extension_id) ON DELETE CASCADE,
    CONSTRAINT fk_team_extensions_user FOREIGN KEY (installed_by) REFERENCES users(uuid),
    -- 唯一约束
    CONSTRAINT uk_team_extensions UNIQUE (team_uuid, extension_id)
);

-- 创建索引
CREATE INDEX idx_team_extensions_team_uuid ON team_extensions(team_uuid);
CREATE INDEX idx_team_extensions_extension_id ON team_extensions(extension_id);

-- 创建更新时间触发器
CREATE TRIGGER update_team_extensions_updated_at BEFORE UPDATE ON team_extensions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

