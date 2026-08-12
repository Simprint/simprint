-- 创建 templates 表
-- 环境配置模板表

CREATE TABLE IF NOT EXISTS templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    user_uuid TEXT NOT NULL,
    team_uuid TEXT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    -- 是否公开（团队内所有人可用）
    is_public INTEGER DEFAULT FALSE,
    -- 摘要
    system_info VARCHAR(100),
    kernel_info VARCHAR(100),
    -- 完整配置（JSON 存储 WindowConfig 结构）
    config_json TEXT NOT NULL DEFAULT '{}',
    -- 使用统计
    usage_count INT DEFAULT 0,
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    -- 约束
    CONSTRAINT fk_templates_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_templates_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid)
);

-- 创建索引
CREATE INDEX idx_templates_user_uuid ON templates(user_uuid);
CREATE INDEX idx_templates_team_uuid ON templates(team_uuid);
CREATE INDEX idx_templates_is_public ON templates(is_public);
CREATE INDEX idx_templates_deleted_at ON templates(deleted_at);

-- 创建更新时间触发器
