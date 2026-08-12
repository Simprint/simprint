-- 创建 proxies 表
-- 代理服务器表

CREATE TABLE IF NOT EXISTS proxies (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INT NOT NULL,
    proxy_type VARCHAR(50) NOT NULL DEFAULT 'http',
    -- 认证信息
    username VARCHAR(255),
    password_encrypted TEXT,
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
    last_checked_at TIMESTAMP WITH TIME ZONE,
    -- 统计
    usage_count INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_proxies_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_proxies_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid)
);

-- 创建索引
CREATE INDEX idx_proxies_user_uuid ON proxies(user_uuid);
CREATE INDEX idx_proxies_team_uuid ON proxies(team_uuid);
CREATE INDEX idx_proxies_proxy_type ON proxies(proxy_type);
CREATE INDEX idx_proxies_status ON proxies(status);
CREATE INDEX idx_proxies_deleted_at ON proxies(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_proxies_updated_at BEFORE UPDATE ON proxies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

