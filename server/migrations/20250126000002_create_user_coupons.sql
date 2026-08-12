-- 创建 user_coupons 表
-- 用户优惠券表

CREATE TABLE IF NOT EXISTS user_coupons (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    coupon_uuid UUID NOT NULL,
    -- 状态: unused, used, expired
    status VARCHAR(50) NOT NULL DEFAULT 'unused',
    -- 发放时间
    issued_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 使用时间
    used_at TIMESTAMP WITH TIME ZONE,
    -- 过期时间（可选，继承自优惠券或自定义）
    expires_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_user_coupons_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_user_coupons_coupon FOREIGN KEY (coupon_uuid) REFERENCES coupons(uuid),
    -- 唯一约束：同一用户不能重复拥有同一优惠券
    UNIQUE(user_uuid, coupon_uuid)
);

-- 创建索引
CREATE INDEX idx_user_coupons_user_uuid ON user_coupons(user_uuid);
CREATE INDEX idx_user_coupons_status ON user_coupons(status);
CREATE INDEX idx_user_coupons_coupon_uuid ON user_coupons(coupon_uuid);
CREATE INDEX idx_user_coupons_expires_at ON user_coupons(expires_at);
