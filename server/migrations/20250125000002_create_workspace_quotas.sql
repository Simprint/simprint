-- 创建 workspace_quotas 表
-- 工作空间配额表，定义工作空间的资源配额限制

CREATE TABLE IF NOT EXISTS workspace_quotas (
    workspace_uuid UUID PRIMARY KEY,
    -- 环境配额
    max_environments INT NOT NULL DEFAULT 10,
    used_environments INT NOT NULL DEFAULT 0,
    -- 团队成员配额（所有团队总和）
    max_team_members INT NOT NULL DEFAULT 5,
    used_team_members INT NOT NULL DEFAULT 0,
    -- 代理配额
    max_proxies INT NOT NULL DEFAULT 10,
    used_proxies INT NOT NULL DEFAULT 0,
    -- RPA 任务配额
    max_rpa_tasks INT NOT NULL DEFAULT 5,
    used_rpa_tasks INT NOT NULL DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_workspace_quotas_workspace FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_workspace_quotas_workspace_uuid ON workspace_quotas(workspace_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_workspace_quotas_updated_at BEFORE UPDATE ON workspace_quotas
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 列注释
COMMENT ON TABLE workspace_quotas IS '工作空间配额表，定义工作空间的资源配额限制';
COMMENT ON COLUMN workspace_quotas.workspace_uuid IS '工作空间 UUID';
COMMENT ON COLUMN workspace_quotas.max_environments IS '最大环境数';
COMMENT ON COLUMN workspace_quotas.used_environments IS '已使用环境数';
COMMENT ON COLUMN workspace_quotas.max_team_members IS '最大成员数（所有团队总和）';
COMMENT ON COLUMN workspace_quotas.used_team_members IS '已使用成员数';
COMMENT ON COLUMN workspace_quotas.max_proxies IS '最大代理数';
COMMENT ON COLUMN workspace_quotas.used_proxies IS '已使用代理数';
COMMENT ON COLUMN workspace_quotas.max_rpa_tasks IS '最大 RPA 任务数';
COMMENT ON COLUMN workspace_quotas.used_rpa_tasks IS '已使用 RPA 任务数';

