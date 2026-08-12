-- 重置推广链接层级默认数据
-- 说明：
-- - 该迁移用于在开发 / 测试环境中统一推广层级配置
-- - 按产品设计保留 4 个层级，按照邀请人数逐级解锁
-- - 为避免脏数据干扰，这里会清空依赖 referral_link_tiers 的相关表

-- 1. 清空依赖表（注意：这会删除所有历史推广数据）
TRUNCATE TABLE redeem_records RESTART IDENTITY CASCADE;
TRUNCATE TABLE referral_rewards RESTART IDENTITY CASCADE;
TRUNCATE TABLE user_referrals RESTART IDENTITY CASCADE;
TRUNCATE TABLE referral_links RESTART IDENTITY CASCADE;
TRUNCATE TABLE referral_link_tiers RESTART IDENTITY CASCADE;

-- 2. 重新插入 4 个默认推广层级
INSERT INTO referral_link_tiers (uuid, name, unlock_threshold, reward_rate, discount_rate, description, sort_order)
VALUES 
    -- 第一档：默认开放
    (gen_random_uuid(), '青铜', 0, 10.00, 5.00, '默认层级，绑定第一个推广链接', 1),
    -- 第二档：达到一定推广人数后解锁
    (gen_random_uuid(), '白银', 10, 15.00, 8.00, '推广满 10 人后解锁第二个推广链接', 2),
    -- 第三档
    (gen_random_uuid(), '黄金', 50, 20.00, 10.00, '推广满 50 人后解锁第三个推广链接', 3),
    -- 第四档
    (gen_random_uuid(), '钻石', 100, 25.00, 15.00, '推广满 100 人后解锁第四个推广链接', 4);

