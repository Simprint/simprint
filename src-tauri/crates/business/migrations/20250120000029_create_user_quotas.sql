-- 创建 user_quotas 表
-- 用户配额表

CREATE TABLE IF NOT EXISTS user_quotas (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL UNIQUE,
    -- 环境配额
    max_environments INT NOT NULL DEFAULT 10,
    used_environments INT NOT NULL DEFAULT 0,
    -- 团队成员配额
    max_team_members INT NOT NULL DEFAULT 5,
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
    CONSTRAINT fk_user_quotas_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_user_quotas_user_uuid ON user_quotas(user_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_user_quotas_updated_at BEFORE UPDATE ON user_quotas
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

