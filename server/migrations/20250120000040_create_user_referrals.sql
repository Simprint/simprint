-- 创建 user_referrals 表
-- 用户邀请关系表

CREATE TABLE IF NOT EXISTS user_referrals (
    id SERIAL PRIMARY KEY,
    -- 邀请者
    inviter_uuid UUID NOT NULL,
    -- 被邀请者
    invitee_uuid UUID NOT NULL UNIQUE,
    -- 通过哪个推广链接
    link_uuid UUID,
    -- 状态: registered, activated, paid
    status VARCHAR(50) NOT NULL DEFAULT 'registered',
    -- 消费统计
    total_consumption DECIMAL(12, 2) DEFAULT 0,
    last_30_days_consumption DECIMAL(12, 2) DEFAULT 0,
    -- 时间
    registered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    activated_at TIMESTAMP WITH TIME ZONE,
    first_paid_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_user_referrals_inviter FOREIGN KEY (inviter_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_user_referrals_invitee FOREIGN KEY (invitee_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_user_referrals_link FOREIGN KEY (link_uuid) REFERENCES referral_links(uuid)
);

-- 创建索引
CREATE INDEX idx_user_referrals_inviter_uuid ON user_referrals(inviter_uuid);
CREATE INDEX idx_user_referrals_invitee_uuid ON user_referrals(invitee_uuid);
CREATE INDEX idx_user_referrals_status ON user_referrals(status);
CREATE INDEX idx_user_referrals_link_uuid ON user_referrals(link_uuid);

