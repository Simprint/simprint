-- 创建 user_wallets 表
-- 用户钱包表

CREATE TABLE IF NOT EXISTS user_wallets (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL UNIQUE,
    -- 余额
    balance DECIMAL(12, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 冻结金额
    frozen_amount DECIMAL(12, 2) NOT NULL DEFAULT 0,
    -- 自动续费组合金额
    auto_renewal_combined DECIMAL(12, 2) NOT NULL DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_user_wallets_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_user_wallets_user_uuid ON user_wallets(user_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_user_wallets_updated_at BEFORE UPDATE ON user_wallets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

