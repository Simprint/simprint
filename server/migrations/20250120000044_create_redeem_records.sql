-- 创建 redeem_records 表
-- 积分兑换记录表

CREATE TABLE IF NOT EXISTS redeem_records (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    option_uuid UUID NOT NULL,
    -- 兑换积分
    points_used INT NOT NULL,
    -- 兑换价值
    value DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    -- 状态: pending, completed, failed
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_redeem_records_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_redeem_records_option FOREIGN KEY (option_uuid) REFERENCES redeem_options(uuid)
);

-- 创建索引
CREATE INDEX idx_redeem_records_user_uuid ON redeem_records(user_uuid);
CREATE INDEX idx_redeem_records_option_uuid ON redeem_records(option_uuid);
CREATE INDEX idx_redeem_records_status ON redeem_records(status);
CREATE INDEX idx_redeem_records_created_at ON redeem_records(created_at);

