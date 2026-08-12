-- 创建 user_preferences 表
-- 用户偏好设置表（仅云同步设置）

CREATE TABLE IF NOT EXISTS user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_uuid TEXT NOT NULL UNIQUE,
    -- 主题: light, dark, system
    theme VARCHAR(50) NOT NULL DEFAULT 'system',
    -- 语言
    language VARCHAR(20) NOT NULL DEFAULT 'zh-CN',
    -- 通知开关
    notifications_enabled INTEGER NOT NULL DEFAULT TRUE,
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_user_preferences_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_user_preferences_user_uuid ON user_preferences(user_uuid);

-- 创建更新时间触发器
