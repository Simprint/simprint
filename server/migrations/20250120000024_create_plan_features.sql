-- 创建 plan_features 表
-- 套餐功能特性表

CREATE TABLE IF NOT EXISTS plan_features (
    id SERIAL PRIMARY KEY,
    plan_uuid UUID NOT NULL,
    -- 特性
    feature_key VARCHAR(100) NOT NULL,
    feature_name VARCHAR(255) NOT NULL,
    feature_value VARCHAR(255),
    -- 是否包含
    is_included BOOLEAN DEFAULT TRUE,
    -- 排序
    sort_order INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_plan_features_plan FOREIGN KEY (plan_uuid) REFERENCES plans(uuid) ON DELETE CASCADE,
    -- 唯一约束
    CONSTRAINT uk_plan_features UNIQUE (plan_uuid, feature_key)
);

-- 创建索引
CREATE INDEX idx_plan_features_plan_uuid ON plan_features(plan_uuid);

