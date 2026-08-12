-- 插入默认数据

-- 1. 默认套餐
INSERT INTO plans (uuid, name, description, price_per_month, price_per_year, max_environments, max_team_members, max_proxies, max_rpa_tasks, is_recommended, sort_order, status)
VALUES 
    (gen_random_uuid(), 'Free', '免费套餐，适合个人试用', 0, 0, 5, 1, 5, 2, FALSE, 1, 'active'),
    (gen_random_uuid(), 'Basic', '基础套餐，适合个人用户', 9.99, 99.99, 20, 3, 20, 5, FALSE, 2, 'active'),
    (gen_random_uuid(), 'Pro', '专业套餐，适合小型团队', 29.99, 299.99, 100, 10, 100, 20, TRUE, 3, 'active'),
    (gen_random_uuid(), 'Enterprise', '企业套餐，无限制', 99.99, 999.99, 1000, 100, 1000, 100, FALSE, 4, 'active')
ON CONFLICT DO NOTHING;

-- 2. 默认推广链接层级
INSERT INTO referral_link_tiers (uuid, name, unlock_threshold, reward_rate, discount_rate, description, sort_order)
VALUES 
    (gen_random_uuid(), '青铜', 0, 10.00, 5.00, '默认层级，邀请即享', 1),
    (gen_random_uuid(), '白银', 10, 15.00, 8.00, '邀请满10人解锁', 2),
    (gen_random_uuid(), '黄金', 50, 20.00, 10.00, '邀请满50人解锁', 3),
    (gen_random_uuid(), '钻石', 100, 25.00, 15.00, '邀请满100人解锁', 4)
ON CONFLICT DO NOTHING;

-- 3. 默认兑换选项
INSERT INTO redeem_options (uuid, redeem_type, name, description, points_required, value, exchange_rate, status, sort_order)
VALUES 
    (gen_random_uuid(), 'wallet', '兑换余额', '100 积分 = $1', 100, 1.00, 100, 'active', 1),
    (gen_random_uuid(), 'coupon', '兑换优惠券', '500 积分 = $10 优惠券', 500, 10.00, 50, 'active', 2),
    (gen_random_uuid(), 'coupon', '兑换优惠券', '1000 积分 = $25 优惠券', 1000, 25.00, 40, 'active', 3)
ON CONFLICT DO NOTHING;

-- 4. 默认系统配置
INSERT INTO system_configs (config_key, config_value, description)
VALUES 
    ('app_version', '"1.0.0"', '应用版本'),
    ('maintenance_mode', 'false', '维护模式'),
    ('registration_enabled', 'true', '是否允许注册'),
    ('referral_enabled', 'true', '是否启用推广计划')
ON CONFLICT (config_key) DO NOTHING;

