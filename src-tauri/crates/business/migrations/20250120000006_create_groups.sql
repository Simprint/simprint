-- 创建 groups 表
-- 环境分组表

CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    user_uuid TEXT NOT NULL,
    team_uuid TEXT,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(50) DEFAULT 'gray',
    sort_order INT DEFAULT 0,
    -- 【关联】分组默认代理（外键在 proxies 表创建后添加）
    default_proxy_uuid TEXT,
    -- 创建者
    created_by TEXT,
    -- 统计字段（计算字段）
    environments_count INT DEFAULT 0,
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    -- 约束
    CONSTRAINT fk_groups_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid),
    CONSTRAINT fk_groups_created_by FOREIGN KEY (created_by) REFERENCES users(uuid)
    -- 注意: default_proxy_uuid 的外键需要在 proxies 表创建后添加
);

-- 创建索引
CREATE INDEX idx_groups_user_uuid ON groups(user_uuid);
CREATE INDEX idx_groups_team_uuid ON groups(team_uuid);
CREATE INDEX idx_groups_deleted_at ON groups(deleted_at);

-- 创建更新时间触发器
