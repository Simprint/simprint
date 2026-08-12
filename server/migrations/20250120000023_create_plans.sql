-- 创建 plans 表
-- 订阅套餐表

CREATE TABLE IF NOT EXISTS plans (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 基础信息
    name VARCHAR(100) NOT NULL,
    description TEXT,
    -- 价格
    price_per_month DECIMAL(12, 2) NOT NULL DEFAULT 0,
    price_per_year DECIMAL(12, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 折扣
    discount_monthly DECIMAL(5, 2) DEFAULT 0,
    discount_yearly DECIMAL(5, 2) DEFAULT 0,
    -- 配额限制
    max_environments INT NOT NULL DEFAULT 10,
    max_team_members INT NOT NULL DEFAULT 5,
    max_proxies INT NOT NULL DEFAULT 10,
    max_rpa_tasks INT NOT NULL DEFAULT 5,
    -- 是否推荐
    is_recommended BOOLEAN DEFAULT FALSE,
    -- 排序
    sort_order INT DEFAULT 0,
    -- 状态: active, deprecated
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_plans_status ON plans(status);
CREATE INDEX idx_plans_sort_order ON plans(sort_order);

-- 创建更新时间触发器
CREATE TRIGGER update_plans_updated_at BEFORE UPDATE ON plans
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

