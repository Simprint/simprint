-- 创建 auto_renewal_services 表
-- 自动续费服务表

CREATE TABLE IF NOT EXISTS auto_renewal_services (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 服务类型: subscription, addon
    service_type VARCHAR(50) NOT NULL,
    -- 关联服务 ID
    service_uuid UUID,
    -- 服务名称
    service_name VARCHAR(255) NOT NULL,
    -- 续费价格
    renewal_price DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 下次扣费日期
    next_bill_date DATE NOT NULL,
    -- 状态: active, paused, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_auto_renewal_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_auto_renewal_services_user_uuid ON auto_renewal_services(user_uuid);
CREATE INDEX idx_auto_renewal_services_status ON auto_renewal_services(status);
CREATE INDEX idx_auto_renewal_services_next_bill_date ON auto_renewal_services(next_bill_date);

-- 创建更新时间触发器
CREATE TRIGGER update_auto_renewal_services_updated_at BEFORE UPDATE ON auto_renewal_services
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

