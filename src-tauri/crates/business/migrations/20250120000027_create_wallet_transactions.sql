-- 创建 wallet_transactions 表
-- 钱包交易记录表

CREATE TABLE IF NOT EXISTS wallet_transactions (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 交易类型: recharge, consume, refund, reward
    transaction_type VARCHAR(50) NOT NULL,
    -- 金额（正数为收入，负数为支出）
    amount DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 余额快照
    balance_before DECIMAL(12, 2) NOT NULL,
    balance_after DECIMAL(12, 2) NOT NULL,
    -- 描述
    description TEXT,
    -- 关联订单
    order_uuid UUID,
    -- 状态: pending, completed, failed
    status VARCHAR(50) NOT NULL DEFAULT 'completed',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_wallet_transactions_user FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_wallet_transactions_user_uuid ON wallet_transactions(user_uuid);
CREATE INDEX idx_wallet_transactions_type ON wallet_transactions(transaction_type);
CREATE INDEX idx_wallet_transactions_created_at ON wallet_transactions(created_at);
CREATE INDEX idx_wallet_transactions_order_uuid ON wallet_transactions(order_uuid);

