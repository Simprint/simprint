-- 创建 platform_accounts 表
-- 平台账号表

CREATE TABLE IF NOT EXISTS platform_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    user_uuid TEXT NOT NULL,
    team_uuid TEXT,
    -- 平台信息
    platform_url VARCHAR(512) NOT NULL,
    platform_name VARCHAR(100),
    -- 账号信息
    account VARCHAR(255) NOT NULL,
    password TEXT,
    -- 状态: active, inactive, expired
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    remark TEXT,
    -- 统计
    usage_count INT DEFAULT 0,
    last_used_at TEXT,
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
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
