-- 创建 team_invitations 表
-- 团队邀请表

CREATE TABLE IF NOT EXISTS team_invitations (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    team_uuid UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    -- 角色
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    -- 邀请者
    invited_by UUID NOT NULL,
    -- 邀请链接
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- 状态: pending, accepted, rejected, expired, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    accepted_at TIMESTAMP WITH TIME ZONE,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_invitations_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_invitations_invited_by FOREIGN KEY (invited_by) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_invitations_team_uuid ON team_invitations(team_uuid);
CREATE INDEX idx_invitations_email ON team_invitations(email);
CREATE INDEX idx_invitations_token ON team_invitations(token);
CREATE INDEX idx_invitations_status ON team_invitations(status);
CREATE INDEX idx_invitations_expires_at ON team_invitations(expires_at);

-- 创建更新时间触发器
CREATE TRIGGER update_team_invitations_updated_at BEFORE UPDATE ON team_invitations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

