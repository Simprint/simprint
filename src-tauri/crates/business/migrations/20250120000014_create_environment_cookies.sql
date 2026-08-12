-- 创建 environment_cookies 表
-- 环境 Cookie 表（环境可导入多个 Cookie）

CREATE TABLE IF NOT EXISTS environment_cookies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_uuid TEXT NOT NULL,
    domain VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    path VARCHAR(255) DEFAULT '/',
    expires_at TEXT,
    http_only INTEGER DEFAULT FALSE,
    secure INTEGER DEFAULT FALSE,
    same_site VARCHAR(20) DEFAULT 'Lax',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_cookies_env FOREIGN KEY (environment_uuid)
        REFERENCES environments(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_env_cookies_env_uuid ON environment_cookies(environment_uuid);
CREATE INDEX idx_env_cookies_domain ON environment_cookies(domain);
