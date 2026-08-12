-- 创建 user_infos 表
-- 用户详细信息表，存储用户的详细业务信息

CREATE TABLE IF NOT EXISTS user_infos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_uuid TEXT NOT NULL UNIQUE,
    nickname VARCHAR(255),
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(50),
    password VARCHAR(255) NOT NULL,
    avatar_hash VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    CONSTRAINT fk_user_infos_user_uuid FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_infos_user_uuid ON user_infos(user_uuid);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_infos_email ON user_infos(email);
CREATE INDEX IF NOT EXISTS idx_user_infos_deleted_at ON user_infos(deleted_at);
CREATE INDEX IF NOT EXISTS idx_user_infos_status ON user_infos(status);

-- 创建更新时间触发器
