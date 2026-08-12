-- 创建 teams 表
-- 团队/工作空间表

CREATE TABLE IF NOT EXISTS teams (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    -- 所有者
    owner_uuid UUID NOT NULL,
    avatar_hash VARCHAR(255),
    -- 配额限制
    max_members INT NOT NULL DEFAULT 10,
    max_environments INT NOT NULL DEFAULT 100,
    max_proxies INT NOT NULL DEFAULT 100,
    -- 【关联】团队默认代理（外键在 proxies 表创建后添加）
    default_proxy_uuid UUID,
    -- 状态
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_teams_owner FOREIGN KEY (owner_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_teams_owner_uuid ON teams(owner_uuid);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_teams_deleted_at ON teams(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

