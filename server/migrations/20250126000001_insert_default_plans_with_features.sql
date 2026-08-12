-- 插入默认套餐数据和特性
-- 删除现有套餐数据（包括关联的特性数据，因为有外键约束会自动删除）

-- 1. 先删除 invoices 表中引用 subscriptions 的数据
-- 由于 invoices 表有外键约束引用 subscriptions，需要先删除发票数据
-- 注意：这会删除所有现有发票记录，请确保这是预期的行为
DELETE FROM invoices;

-- 2. 删除 subscriptions 表中引用 plans 的数据
-- 由于 subscriptions 表有外键约束（没有 ON DELETE CASCADE），需要先删除订阅数据
-- 注意：这会删除所有现有订阅记录，请确保这是预期的行为
DELETE FROM subscriptions;

-- 3. 删除现有套餐数据（会级联删除 plan_features 数据）
DELETE FROM plans;

-- 4. 插入新的套餐数据
-- 套餐：免费套餐、基础套餐、专业套餐、商业套餐、企业套餐
-- 月付价格：0, 21.4, 11.4, 758.4, 3558.4
-- 年付价格：按月付*10计算（相当于2个月免费）
-- 最大环境：8, 100, 1000, 10000, 100000
-- 最大团队：1, 5, 10, 18, 30
-- 代理数量：所有都是99999
-- max_rpa_tasks：所有都是99999

INSERT INTO plans (uuid, name, description, price_per_month, price_per_year, currency, max_environments, max_team_members, max_proxies, max_rpa_tasks, is_recommended, sort_order, status)
VALUES 
    -- 免费套餐：月付 0，年付 0，环境 8，团队 1
    (gen_random_uuid(), '免费套餐', '适合个人试用，体验基础功能', 0.00, 0.00, 'USD', 8, 1, 99999, 99999, FALSE, 1, 'active'),
    -- 基础套餐：月付 21.4，年付 214.00，环境 100，团队 5
    (gen_random_uuid(), '基础套餐', '适合个人用户和小型项目', 21.40, 214.00, 'USD', 100, 5, 99999, 99999, FALSE, 2, 'active'),
    -- 专业套餐：月付 11.4，年付 114.00，环境 1000，团队 10
    (gen_random_uuid(), '专业套餐', '适合小型团队和成长型企业', 108.40, 1084.00, 'USD', 1000, 10, 99999, 99999, TRUE, 3, 'active'),
    -- 商业套餐：月付 758.4，年付 7584.00，环境 10000，团队 18
    (gen_random_uuid(), '商业套餐', '适合中大型企业和专业团队', 758.40, 7584.00, 'USD', 10000, 18, 99999, 99999, FALSE, 4, 'active'),
    -- 企业套餐：月付 3558.4，年付 35584.00，环境 100000，团队 30
    (gen_random_uuid(), '企业套餐', '适合大型企业，提供最高级别的服务和支持', 3558.40, 35584.00, 'USD', 100000, 30, 99999, 99999, FALSE, 5, 'active')
ON CONFLICT DO NOTHING;

-- 5. 为每个套餐插入特性数据
-- 使用 CTE 来获取套餐 UUID，然后插入特性

-- 免费套餐特性
INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'env_limit',
    '环境数量',
    '8 个环境',
    TRUE,
    1
FROM plans p WHERE p.name = '免费套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'team_members',
    '团队成员',
    '1 人',
    TRUE,
    2
FROM plans p WHERE p.name = '免费套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'basic_support',
    '基础支持',
    '社区支持',
    TRUE,
    3
FROM plans p WHERE p.name = '免费套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

-- 基础套餐特性
INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'env_limit',
    '环境数量',
    '100 个环境',
    TRUE,
    1
FROM plans p WHERE p.name = '基础套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'team_members',
    '团队成员',
    '5 人',
    TRUE,
    2
FROM plans p WHERE p.name = '基础套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'proxy_access',
    '代理访问',
    '99999 个代理',
    TRUE,
    3
FROM plans p WHERE p.name = '基础套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'rpa_tasks',
    'RPA 任务',
    '99999 个任务',
    TRUE,
    4
FROM plans p WHERE p.name = '基础套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'email_support',
    '邮件支持',
    '工作日支持',
    TRUE,
    5
FROM plans p WHERE p.name = '基础套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

-- 专业套餐特性（包含基础套餐的所有特性，并增加更多）
INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'env_limit',
    '环境数量',
    '1000 个环境',
    TRUE,
    1
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'team_members',
    '团队成员',
    '10 人',
    TRUE,
    2
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'proxy_access',
    '代理访问',
    '99999 个代理',
    TRUE,
    3
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'rpa_tasks',
    'RPA 任务',
    '99999 个任务',
    TRUE,
    4
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'priority_support',
    '优先支持',
    '工作日优先响应',
    TRUE,
    5
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'api_access',
    'API 访问',
    '完整 API 权限',
    TRUE,
    6
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'advanced_features',
    '高级功能',
    '指纹管理、Cookie 同步等',
    TRUE,
    7
FROM plans p WHERE p.name = '专业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

-- 商业套餐特性（包含专业套餐的所有特性，并增加更多）
INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'env_limit',
    '环境数量',
    '10000 个环境',
    TRUE,
    1
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'team_members',
    '团队成员',
    '18 人',
    TRUE,
    2
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'proxy_access',
    '代理访问',
    '99999 个代理',
    TRUE,
    3
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'rpa_tasks',
    'RPA 任务',
    '99999 个任务',
    TRUE,
    4
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'dedicated_support',
    '专属支持',
    '7x24 小时专属支持',
    TRUE,
    5
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'api_access',
    'API 访问',
    '完整 API 权限 + 高级 API',
    TRUE,
    6
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'advanced_features',
    '高级功能',
    '所有高级功能',
    TRUE,
    7
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'custom_integration',
    '定制集成',
    '支持定制化集成',
    TRUE,
    8
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'sla_guarantee',
    'SLA 保障',
    '99.9% 可用性保障',
    TRUE,
    9
FROM plans p WHERE p.name = '商业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

-- 企业套餐特性（包含商业套餐的所有特性，并增加更多）
INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'env_limit',
    '环境数量',
    '100000 个环境',
    TRUE,
    1
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'team_members',
    '团队成员',
    '30 人',
    TRUE,
    2
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'proxy_access',
    '代理访问',
    '99999 个代理',
    TRUE,
    3
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'rpa_tasks',
    'RPA 任务',
    '99999 个任务',
    TRUE,
    4
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'premium_support',
    '高级支持',
    '7x24 小时专属支持 + 专属客户经理',
    TRUE,
    5
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'api_access',
    'API 访问',
    '完整 API 权限 + 高级 API + 定制 API',
    TRUE,
    6
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'advanced_features',
    '高级功能',
    '所有高级功能 + 企业级功能',
    TRUE,
    7
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'custom_integration',
    '定制集成',
    '完全定制化集成 + 专属技术支持',
    TRUE,
    8
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'sla_guarantee',
    'SLA 保障',
    '99.99% 可用性保障',
    TRUE,
    9
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'dedicated_infrastructure',
    '专属基础设施',
    '专属服务器和资源',
    TRUE,
    10
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

INSERT INTO plan_features (plan_uuid, feature_key, feature_name, feature_value, is_included, sort_order)
SELECT 
    p.uuid,
    'training_sessions',
    '培训服务',
    '定期培训和最佳实践指导',
    TRUE,
    11
FROM plans p WHERE p.name = '企业套餐'
ON CONFLICT (plan_uuid, feature_key) DO NOTHING;

