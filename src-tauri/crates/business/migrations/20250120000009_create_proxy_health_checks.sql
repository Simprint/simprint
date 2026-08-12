-- 创建 proxy_health_checks 表
-- 代理健康检查记录表

CREATE TABLE IF NOT EXISTS proxy_health_checks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proxy_uuid TEXT NOT NULL,
    -- 检查结果
    status VARCHAR(50) NOT NULL,
    latency INT,
    ip_address VARCHAR(45),
    error_message TEXT,
    -- 时间
    checked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_proxy_health_checks_proxy FOREIGN KEY (proxy_uuid)
        REFERENCES proxies(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_proxy_health_checks_proxy_uuid ON proxy_health_checks(proxy_uuid);
CREATE INDEX idx_proxy_health_checks_checked_at ON proxy_health_checks(checked_at);
CREATE INDEX idx_proxy_health_checks_status ON proxy_health_checks(status);
