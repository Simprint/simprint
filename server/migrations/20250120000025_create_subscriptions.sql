-- 创建 subscriptions 表
-- 用户订阅表

CREATE TABLE IF NOT EXISTS subscriptions (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    plan_uuid UUID NOT NULL,
    -- 订阅周期: monthly, yearly
    billing_period VARCHAR(20) NOT NULL DEFAULT 'monthly',
    -- 价格快照
    price DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 时间
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    next_billing_date DATE,
    -- 自动续费
    auto_renew BOOLEAN DEFAULT TRUE,
    -- 状态: active, cancelled, expired, suspended
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    cancelled_at TIMESTAMP WITH TIME ZONE,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_subscriptions_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_subscriptions_plan FOREIGN KEY (plan_uuid) REFERENCES plans(uuid)
);

-- 创建索引
CREATE INDEX idx_subscriptions_user_uuid ON subscriptions(user_uuid);
CREATE INDEX idx_subscriptions_plan_uuid ON subscriptions(plan_uuid);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_expires_at ON subscriptions(expires_at);

-- 创建更新时间触发器
CREATE TRIGGER update_subscriptions_updated_at BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

