-- 创建 referral_links 表
-- 用户推广链接表

CREATE TABLE IF NOT EXISTS referral_links (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 链接码
    code VARCHAR(50) NOT NULL UNIQUE,
    -- 完整 URL
    url VARCHAR(512),
    -- 当前层级
    tier_uuid UUID,
    -- 是否解锁
    unlocked BOOLEAN DEFAULT TRUE,
    -- 是否当前使用
    is_current BOOLEAN DEFAULT FALSE,
    -- 奖励比例（冗余，便于查询）
    reward_rate DECIMAL(5, 2) NOT NULL DEFAULT 0,
    discount_rate DECIMAL(5, 2) NOT NULL DEFAULT 0,
    -- 统计
    registered_users INT DEFAULT 0,
    paid_users INT DEFAULT 0,
    total_consumption DECIMAL(12, 2) DEFAULT 0,
    last_30_days_consumption DECIMAL(12, 2) DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_referral_links_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_referral_links_tier FOREIGN KEY (tier_uuid) REFERENCES referral_link_tiers(uuid)
);

-- 创建索引
CREATE INDEX idx_referral_links_user_uuid ON referral_links(user_uuid);
CREATE INDEX idx_referral_links_code ON referral_links(code);
CREATE INDEX idx_referral_links_is_current ON referral_links(is_current);

-- 创建更新时间触发器
CREATE TRIGGER update_referral_links_updated_at BEFORE UPDATE ON referral_links
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

