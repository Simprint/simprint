-- 创建 workspaces 表
-- 工作空间表，资源隔离的顶层容器

CREATE TABLE IF NOT EXISTS workspaces (
    uuid TEXT PRIMARY KEY DEFAULT (randomblob(16)),
    name VARCHAR(255) NOT NULL,
    owner_uuid TEXT NOT NULL,
    workspace_type VARCHAR(50) NOT NULL DEFAULT 'personal',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    -- 约束
    CONSTRAINT fk_workspaces_owner FOREIGN KEY (owner_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_workspaces_owner_uuid ON workspaces(owner_uuid);
CREATE INDEX idx_workspaces_deleted_at ON workspaces(deleted_at);
CREATE INDEX idx_workspaces_workspace_type ON workspaces(workspace_type);

-- 创建更新时间触发器

-- 列注释
