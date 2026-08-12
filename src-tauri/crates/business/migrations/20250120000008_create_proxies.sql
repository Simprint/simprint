-- 创建 proxies 表
-- 代理服务器表

CREATE TABLE IF NOT EXISTS proxies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    user_uuid TEXT,
    team_uuid TEXT,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INT NOT NULL,
    proxy_type VARCHAR(50) NOT NULL DEFAULT 'http',
    -- 认证信息
    username VARCHAR(255),
    password TEXT,
    -- SSH 类型额外字段
    ssh_key_encrypted TEXT,
    ssh_passphrase_encrypted TEXT,
    -- 地理位置信息
    country VARCHAR(100),
    city VARCHAR(100),
    -- 状态
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    latency INT,
    last_check_ip VARCHAR(45),
    last_checked_at TEXT,
    -- 统计
    usage_count INT DEFAULT 0,
    -- 时间
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    -- 约束
    CONSTRAINT fk_proxies_user FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_proxies_user_uuid ON proxies(user_uuid);
CREATE INDEX idx_proxies_team_uuid ON proxies(team_uuid);
CREATE INDEX idx_proxies_proxy_type ON proxies(proxy_type);
CREATE INDEX idx_proxies_status ON proxies(status);
CREATE INDEX idx_proxies_deleted_at ON proxies(deleted_at);

-- 创建更新时间触发器
