-- 创建 environment_accounts 表
-- 环境-账号关联表（多对多）

CREATE TABLE IF NOT EXISTS environment_accounts (
    id SERIAL PRIMARY KEY,
    environment_uuid UUID NOT NULL,
    account_uuid UUID NOT NULL,
    sort_order INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_accounts_env FOREIGN KEY (environment_uuid) 
        REFERENCES environments(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_env_accounts_account FOREIGN KEY (account_uuid) 
        REFERENCES platform_accounts(uuid) ON DELETE CASCADE,
    -- 唯一约束：同一环境不能重复关联同一账号
    CONSTRAINT uk_env_accounts UNIQUE (environment_uuid, account_uuid)
);

-- 创建索引
CREATE INDEX idx_env_accounts_env_uuid ON environment_accounts(environment_uuid);
CREATE INDEX idx_env_accounts_account_uuid ON environment_accounts(account_uuid);

