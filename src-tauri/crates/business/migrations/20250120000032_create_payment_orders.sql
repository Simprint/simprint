-- 创建 payment_orders 表
-- 支付订单表

CREATE TABLE IF NOT EXISTS payment_orders (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 订单号
    order_no VARCHAR(100) NOT NULL UNIQUE,
    user_uuid UUID NOT NULL,
    -- 订单类型: recharge, subscription, addon, refund
    order_type VARCHAR(50) NOT NULL,
    -- 金额
    amount DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 状态: pending, paid, failed, refunded, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 支付渠道: alipay, wechatpay, stripe, paypal 等
    payment_channel VARCHAR(50),
    -- 第三方订单号
    external_order_id VARCHAR(255),
    -- 描述
    description TEXT,
    -- 关联
    subscription_uuid UUID,
    coupon_uuid UUID,
    -- 折扣信息
    original_amount DECIMAL(12, 2),
    discount_amount DECIMAL(12, 2) DEFAULT 0,
    -- 时间
    paid_at TIMESTAMP WITH TIME ZONE,
    refunded_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_payment_orders_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_payment_orders_subscription FOREIGN KEY (subscription_uuid) REFERENCES subscriptions(uuid),
    CONSTRAINT fk_payment_orders_coupon FOREIGN KEY (coupon_uuid) REFERENCES coupons(uuid)
);

-- 创建索引
CREATE INDEX idx_payment_orders_user_uuid ON payment_orders(user_uuid);
CREATE INDEX idx_payment_orders_order_no ON payment_orders(order_no);
CREATE INDEX idx_payment_orders_status ON payment_orders(status);
CREATE INDEX idx_payment_orders_order_type ON payment_orders(order_type);
CREATE INDEX idx_payment_orders_created_at ON payment_orders(created_at);

-- 创建更新时间触发器
CREATE TRIGGER update_payment_orders_updated_at BEFORE UPDATE ON payment_orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

