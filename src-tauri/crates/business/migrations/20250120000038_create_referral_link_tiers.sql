-- 创建 referral_link_tiers 表
-- 推广链接层级配置表

CREATE TABLE IF NOT EXISTS referral_link_tiers (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 层级名称
    name VARCHAR(100) NOT NULL,
    -- 解锁条件（邀请人数）
    unlock_threshold INT NOT NULL DEFAULT 0,
    -- 奖励比例（百分比）
    reward_rate DECIMAL(5, 2) NOT NULL DEFAULT 0,
    -- 被邀请人折扣比例（百分比）
    discount_rate DECIMAL(5, 2) NOT NULL DEFAULT 0,
    -- 描述
    description TEXT,
    -- 排序
    sort_order INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_referral_link_tiers_unlock ON referral_link_tiers(unlock_threshold);
CREATE INDEX idx_referral_link_tiers_sort ON referral_link_tiers(sort_order);

-- 创建更新时间触发器
CREATE TRIGGER update_referral_link_tiers_updated_at BEFORE UPDATE ON referral_link_tiers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

