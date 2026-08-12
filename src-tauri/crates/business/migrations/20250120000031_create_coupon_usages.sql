-- 创建 coupon_usages 表
-- 优惠券使用记录表

CREATE TABLE IF NOT EXISTS coupon_usages (
    id SERIAL PRIMARY KEY,
    coupon_uuid UUID NOT NULL,
    user_uuid UUID NOT NULL,
    order_uuid UUID,
    -- 折扣金额
    discount_amount DECIMAL(12, 2) NOT NULL,
    -- 使用时间
    used_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_coupon_usages_coupon FOREIGN KEY (coupon_uuid) REFERENCES coupons(uuid),
    CONSTRAINT fk_coupon_usages_user FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_coupon_usages_coupon_uuid ON coupon_usages(coupon_uuid);
CREATE INDEX idx_coupon_usages_user_uuid ON coupon_usages(user_uuid);

