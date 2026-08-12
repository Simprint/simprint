-- 创建 workspaces 表
-- 工作空间表，资源隔离的顶层容器

CREATE TABLE IF NOT EXISTS workspaces (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    owner_uuid UUID NOT NULL,
    workspace_type VARCHAR(50) NOT NULL DEFAULT 'personal',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_workspaces_owner FOREIGN KEY (owner_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_workspaces_owner_uuid ON workspaces(owner_uuid);
CREATE INDEX idx_workspaces_deleted_at ON workspaces(deleted_at);
CREATE INDEX idx_workspaces_workspace_type ON workspaces(workspace_type);

-- 创建更新时间触发器
CREATE TRIGGER update_workspaces_updated_at BEFORE UPDATE ON workspaces
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 列注释
COMMENT ON TABLE workspaces IS '工作空间表，资源隔离的顶层容器';
COMMENT ON COLUMN workspaces.uuid IS '工作空间唯一标识';
COMMENT ON COLUMN workspaces.name IS '工作空间名称';
COMMENT ON COLUMN workspaces.owner_uuid IS '所有者用户 UUID';
COMMENT ON COLUMN workspaces.workspace_type IS '工作空间类型：personal/team/enterprise';
COMMENT ON COLUMN workspaces.created_at IS '创建时间';
COMMENT ON COLUMN workspaces.updated_at IS '更新时间';
COMMENT ON COLUMN workspaces.deleted_at IS '删除时间（软删除）';

