-- 创建 group_member_permissions 表
-- 分组权限表，控制团队成员对分组的访问权限

CREATE TABLE IF NOT EXISTS group_member_permissions (
    group_uuid UUID NOT NULL,
    workspace_uuid UUID NOT NULL,
    team_uuid UUID NOT NULL,
    user_uuid UUID NOT NULL,
    permission_type VARCHAR(50) NOT NULL DEFAULT 'read',
    granted_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_group_member_permissions_group FOREIGN KEY (group_uuid) REFERENCES groups(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_group_member_permissions_workspace FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_group_member_permissions_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_group_member_permissions_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_group_member_permissions_granted_by FOREIGN KEY (granted_by) REFERENCES users(uuid),
    -- 唯一约束：一个用户对一个分组只能有一条权限记录
    CONSTRAINT uk_group_member_permissions UNIQUE (group_uuid, user_uuid),
    -- 检查约束：权限类型必须是 read/write/manage 之一
    CONSTRAINT ck_group_member_permissions_type CHECK (permission_type IN ('read', 'write', 'manage'))
);

-- 创建索引
CREATE INDEX idx_group_member_permissions_group_uuid ON group_member_permissions(group_uuid);
CREATE INDEX idx_group_member_permissions_user_uuid ON group_member_permissions(user_uuid);
CREATE INDEX idx_group_member_permissions_workspace_uuid ON group_member_permissions(workspace_uuid);
CREATE INDEX idx_group_member_permissions_team_uuid ON group_member_permissions(team_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_group_member_permissions_updated_at BEFORE UPDATE ON group_member_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 列注释
COMMENT ON TABLE group_member_permissions IS '分组权限表，控制团队成员对分组的访问权限';
COMMENT ON COLUMN group_member_permissions.group_uuid IS '分组 UUID';
COMMENT ON COLUMN group_member_permissions.workspace_uuid IS '工作空间 UUID（冗余，便于查询）';
COMMENT ON COLUMN group_member_permissions.team_uuid IS '团队 UUID（冗余，便于查询）';
COMMENT ON COLUMN group_member_permissions.user_uuid IS '用户 UUID';
COMMENT ON COLUMN group_member_permissions.permission_type IS '权限类型：read/write/manage';
COMMENT ON COLUMN group_member_permissions.granted_by IS '授权者 UUID';
COMMENT ON COLUMN group_member_permissions.created_at IS '创建时间';
COMMENT ON COLUMN group_member_permissions.updated_at IS '更新时间';

