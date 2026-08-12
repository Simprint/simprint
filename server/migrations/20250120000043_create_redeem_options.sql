-- 创建 redeem_options 表
-- 积分兑换选项表

CREATE TABLE IF NOT EXISTS redeem_options (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 兑换类型: wallet, coupon, gift
    redeem_type VARCHAR(50) NOT NULL,
    -- 名称
    name VARCHAR(255) NOT NULL,
    -- 描述
    description TEXT,
    -- 所需积分
    points_required INT NOT NULL,
    -- 兑换价值
    value DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) DEFAULT 'USD',
    -- 兑换比例（每 N 积分 = 1 单位价值）
    exchange_rate INT NOT NULL DEFAULT 100,
    -- 状态: active, inactive
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 排序
    sort_order INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_redeem_options_redeem_type ON redeem_options(redeem_type);
CREATE INDEX idx_redeem_options_status ON redeem_options(status);

-- 创建更新时间触发器
CREATE TRIGGER update_redeem_options_updated_at BEFORE UPDATE ON redeem_options
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

