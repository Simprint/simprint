-- 为优惠券表添加名称和描述字段

ALTER TABLE coupons
ADD COLUMN IF NOT EXISTS name VARCHAR(100),
ADD COLUMN IF NOT EXISTS description TEXT;

-- 为现有优惠券更新名称和描述
UPDATE coupons SET name = '新用户专享', description = '新用户注册专享优惠，享受10%折扣' WHERE code = 'WELCOME10';
UPDATE coupons SET name = '年付优惠', description = '年付订阅专享，立减$20，最低消费$100' WHERE code = 'YEARLY20';
UPDATE coupons SET name = '限时促销', description = '限时促销活动，享受15%折扣，最大折扣$50' WHERE code = 'PROMO15';
UPDATE coupons SET name = '大额订单优惠', description = '大额订单专享，立减$50，最低消费$200' WHERE code = 'BIG50';
UPDATE coupons SET name = '年付专属', description = '年付订阅专属优惠，享受20%折扣，最大折扣$200' WHERE code = 'YEARLY20PCT';
UPDATE coupons SET name = '首次购买', description = '首次购买专享，立减$10，无最低消费限制' WHERE code = 'FIRST10';
