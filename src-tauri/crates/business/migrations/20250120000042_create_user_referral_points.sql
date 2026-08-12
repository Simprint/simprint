-- 创建 user_referral_points 表
-- 用户推荐积分表

CREATE TABLE IF NOT EXISTS user_referral_points (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL UNIQUE,
    -- 总积分
    total_points INT NOT NULL DEFAULT 0,
    -- 可用积分
    available_points INT NOT NULL DEFAULT 0,
    -- 已使用积分
    used_points INT NOT NULL DEFAULT 0,
    -- 待审核积分
    pending_points INT NOT NULL DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_user_referral_points_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_user_referral_points_user_uuid ON user_referral_points(user_uuid);

-- 创建更新时间触发器
CREATE TRIGGER update_user_referral_points_updated_at BEFORE UPDATE ON user_referral_points
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

