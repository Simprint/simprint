-- 创建 login_history 表
-- 登录历史表（安全审计）

CREATE TABLE IF NOT EXISTS login_history (
    id BIGSERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    -- 登录信息
    ip_address VARCHAR(45) NOT NULL,
    device_info VARCHAR(255),
    user_agent TEXT,
    -- 位置信息（通过 IP 解析）
    location VARCHAR(255),
    country VARCHAR(100),
    city VARCHAR(100),
    -- 结果
    success BOOLEAN NOT NULL DEFAULT TRUE,
    failure_reason VARCHAR(255),
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_login_history_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_login_history_user_uuid ON login_history(user_uuid);
CREATE INDEX idx_login_history_created_at ON login_history(created_at);
CREATE INDEX idx_login_history_ip ON login_history(ip_address);
CREATE INDEX idx_login_history_success ON login_history(success);

