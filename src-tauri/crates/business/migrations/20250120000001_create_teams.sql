-- 创建 teams 表
-- 团队/工作空间表

CREATE TABLE IF NOT EXISTS teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    -- 所有者
    owner_uuid TEXT NOT NULL,
    avatar_hash VARCHAR(255),
    -- 配额限制
    max_members INT NOT NULL DEFAULT 10,
    max_environments INT NOT NULL DEFAULT 100,
    max_proxies INT NOT NULL DEFAULT 100,
    -- 【关联】团队默认代理（外键在 proxies 表创建后添加）
    default_proxy_uuid TEXT,
    -- 状态
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    -- 约束
    CONSTRAINT fk_teams_owner FOREIGN KEY (owner_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_teams_owner_uuid ON teams(owner_uuid);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_teams_deleted_at ON teams(deleted_at);

-- 创建更新时间触发器
