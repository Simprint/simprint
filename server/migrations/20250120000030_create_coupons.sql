-- 创建 coupons 表
-- 优惠券表

CREATE TABLE IF NOT EXISTS coupons (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 优惠券码
    code VARCHAR(50) NOT NULL UNIQUE,
    -- 优惠类型: percent, fixed
    discount_type VARCHAR(20) NOT NULL,
    -- 优惠值（百分比或固定金额）
    discount_value DECIMAL(12, 2) NOT NULL,
    -- 最低消费
    min_amount DECIMAL(12, 2) DEFAULT 0,
    -- 最大折扣（百分比类型时有效）
    max_discount DECIMAL(12, 2),
    -- 使用限制
    max_uses INT,
    used_count INT NOT NULL DEFAULT 0,
    -- 每用户限制
    max_uses_per_user INT DEFAULT 1,
    -- 有效期
    valid_from TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMP WITH TIME ZONE,
    -- 适用范围: all, subscription, addon
    applicable_to VARCHAR(50) NOT NULL DEFAULT 'all',
    -- 状态: active, inactive, expired
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_coupons_code ON coupons(code);
CREATE INDEX idx_coupons_status ON coupons(status);
CREATE INDEX idx_coupons_valid_until ON coupons(valid_until);

-- 创建更新时间触发器
CREATE TRIGGER update_coupons_updated_at BEFORE UPDATE ON coupons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

