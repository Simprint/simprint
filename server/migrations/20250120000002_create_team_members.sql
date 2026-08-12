-- 创建 team_members 表
-- 团队成员关联表（用户 ↔ 团队，多对多）

CREATE TABLE IF NOT EXISTS team_members (
    id SERIAL PRIMARY KEY,
    team_uuid UUID NOT NULL,
    user_uuid UUID NOT NULL,
    -- 角色: owner, admin, editor, viewer
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    -- 加入信息
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invited_by UUID,
    -- 统计字段（冗余，提高查询性能）
    environment_count INT NOT NULL DEFAULT 0,
    group_count INT NOT NULL DEFAULT 0,
    -- 状态
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_team_members_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_team_members_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_team_members_invited_by FOREIGN KEY (invited_by) REFERENCES users(uuid),
    -- 唯一约束：用户在同一团队只能有一条记录
    CONSTRAINT uk_team_members UNIQUE (team_uuid, user_uuid)
);

-- 创建索引
CREATE INDEX idx_team_members_team_uuid ON team_members(team_uuid);
CREATE INDEX idx_team_members_user_uuid ON team_members(user_uuid);
CREATE INDEX idx_team_members_role ON team_members(role);
CREATE INDEX idx_team_members_status ON team_members(status);

-- 创建更新时间触发器
CREATE TRIGGER update_team_members_updated_at BEFORE UPDATE ON team_members
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

