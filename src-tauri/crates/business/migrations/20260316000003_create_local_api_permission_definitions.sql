-- 创建本地 API 权限定义表

CREATE TABLE IF NOT EXISTS local_api_permission_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL DEFAULT (randomblob(16)) UNIQUE,
    permission_code VARCHAR(128) NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    description TEXT,
    default_enabled INTEGER NOT NULL DEFAULT TRUE,
    default_rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    default_rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    CONSTRAINT chk_local_api_permission_definitions_minute CHECK (default_rate_limit_per_minute >= 0),
    CONSTRAINT chk_local_api_permission_definitions_hour CHECK (default_rate_limit_per_hour >= 0)
);

CREATE INDEX idx_local_api_permission_definitions_sort_order
    ON local_api_permission_definitions(sort_order);
