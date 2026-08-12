-- 创建 users 表
-- 用户基础信息表，存储用户的基础标识信息

CREATE TABLE IF NOT EXISTS users (
    uuid TEXT PRIMARY KEY DEFAULT (randomblob(16)),
    id VARCHAR(255) NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);

-- 创建更新时间触发器
