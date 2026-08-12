-- 创建 referral_rewards 表
-- 推荐奖励记录表

CREATE TABLE IF NOT EXISTS referral_rewards (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 奖励类型: registration, first_pay, consumption
    reward_type VARCHAR(50) NOT NULL,
    -- 积分
    points INT NOT NULL,
    -- 描述
    description TEXT,
    -- 关联被邀请用户
    referred_user_uuid UUID,
    -- 关联推广链接
    link_uuid UUID,
    -- 状态: pending, approved, rejected
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_referral_rewards_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_referral_rewards_referred FOREIGN KEY (referred_user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_referral_rewards_link FOREIGN KEY (link_uuid) REFERENCES referral_links(uuid)
);

-- 创建索引
CREATE INDEX idx_referral_rewards_user_uuid ON referral_rewards(user_uuid);
CREATE INDEX idx_referral_rewards_reward_type ON referral_rewards(reward_type);
CREATE INDEX idx_referral_rewards_status ON referral_rewards(status);
CREATE INDEX idx_referral_rewards_created_at ON referral_rewards(created_at);

