-- 创建 platform_accounts 表
-- 平台账号表

CREATE TABLE IF NOT EXISTS platform_accounts (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 平台信息
    platform_url VARCHAR(512) NOT NULL,
    platform_name VARCHAR(100),
    -- 账号信息
    account VARCHAR(255) NOT NULL,
    password_encrypted TEXT,
    -- 状态: active, inactive, expired
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    remark TEXT,
    -- 统计
    usage_count INT DEFAULT 0,
    last_used_at TIMESTAMP WITH TIME ZONE,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_platform_accounts_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_platform_accounts_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid)
);

-- 创建索引
CREATE INDEX idx_platform_accounts_user_uuid ON platform_accounts(user_uuid);
CREATE INDEX idx_platform_accounts_team_uuid ON platform_accounts(team_uuid);
CREATE INDEX idx_platform_accounts_platform_name ON platform_accounts(platform_name);
CREATE INDEX idx_platform_accounts_status ON platform_accounts(status);
CREATE INDEX idx_platform_accounts_deleted_at ON platform_accounts(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_platform_accounts_updated_at BEFORE UPDATE ON platform_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

