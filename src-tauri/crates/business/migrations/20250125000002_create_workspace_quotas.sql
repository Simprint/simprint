-- 创建 workspace_quotas 表
-- 工作空间配额表，定义工作空间的资源配额限制

CREATE TABLE IF NOT EXISTS workspace_quotas (
    workspace_uuid TEXT PRIMARY KEY,
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
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_workspace_quotas_workspace FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_workspace_quotas_workspace_uuid ON workspace_quotas(workspace_uuid);

-- 创建更新时间触发器

-- 列注释
