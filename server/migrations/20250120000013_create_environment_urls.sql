-- 创建 environment_urls 表
-- 环境预设 URL 表（环境可有多个预设 URL）

CREATE TABLE IF NOT EXISTS environment_urls (
    id SERIAL PRIMARY KEY,
    environment_uuid UUID NOT NULL,
    url VARCHAR(2048) NOT NULL,
    title VARCHAR(255),
    sort_order INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_urls_env FOREIGN KEY (environment_uuid) 
        REFERENCES environments(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_env_urls_env_uuid ON environment_urls(environment_uuid);

