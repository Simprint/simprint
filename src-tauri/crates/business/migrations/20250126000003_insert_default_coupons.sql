-- 插入默认优惠券数据

-- 1. 新用户专享优惠券（10%折扣，无使用次数限制，适用于所有订单）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'WELCOME10', 'percentage', 10.00, 0.00, NULL, NULL, 1, CURRENT_TIMESTAMP, NULL, 'all', 'active')
ON CONFLICT (code) DO NOTHING;

-- 2. 年付优惠券（固定金额 $20，最低消费 $100，适用于订阅）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'YEARLY20', 'fixed', 20.00, 100.00, NULL, NULL, 1, CURRENT_TIMESTAMP, NULL, 'subscription', 'active')
ON CONFLICT (code) DO NOTHING;

-- 3. 限时促销优惠券（15%折扣，最大折扣 $50，使用次数限制 100 次）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'PROMO15', 'percentage', 15.00, 50.00, 50.00, 100, 1, CURRENT_TIMESTAMP, (CURRENT_TIMESTAMP + INTERVAL '30 days'), 'all', 'active')
ON CONFLICT (code) DO NOTHING;

-- 4. 大额订单优惠券（固定金额 $50，最低消费 $200）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'BIG50', 'fixed', 50.00, 200.00, NULL, NULL, 1, CURRENT_TIMESTAMP, NULL, 'all', 'active')
ON CONFLICT (code) DO NOTHING;

-- 5. 年付专属优惠券（20%折扣，仅适用于年付订阅，最大折扣 $200）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'YEARLY20PCT', 'percentage', 20.00, 100.00, 200.00, NULL, 1, CURRENT_TIMESTAMP, NULL, 'subscription', 'active')
ON CONFLICT (code) DO NOTHING;

-- 6. 首次购买优惠券（固定金额 $10，无最低消费，每人限用1次）
INSERT INTO coupons (uuid, code, discount_type, discount_value, min_amount, max_discount, max_uses, max_uses_per_user, valid_from, valid_until, applicable_to, status)
VALUES 
    (gen_random_uuid(), 'FIRST10', 'fixed', 10.00, 0.00, NULL, NULL, 1, CURRENT_TIMESTAMP, NULL, 'all', 'active')
ON CONFLICT (code) DO NOTHING;
