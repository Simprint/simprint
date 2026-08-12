-- 创建 invoices 表
-- 发票表

CREATE TABLE IF NOT EXISTS invoices (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    -- 发票号
    invoice_number VARCHAR(100) NOT NULL UNIQUE,
    -- 金额
    amount DECIMAL(12, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    -- 关联订阅/订单
    subscription_uuid UUID,
    order_uuid UUID,
    -- 发票类型: subscription, addon, recharge
    invoice_type VARCHAR(50) NOT NULL,
    -- 状态: draft, issued, paid, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    -- 时间
    issued_at TIMESTAMP WITH TIME ZONE,
    due_at TIMESTAMP WITH TIME ZONE,
    paid_at TIMESTAMP WITH TIME ZONE,
    -- 发票信息（PDF 链接等）
    invoice_url VARCHAR(512),
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_invoices_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_invoices_subscription FOREIGN KEY (subscription_uuid) REFERENCES subscriptions(uuid)
);

-- 创建索引
CREATE INDEX idx_invoices_user_uuid ON invoices(user_uuid);
CREATE INDEX idx_invoices_invoice_number ON invoices(invoice_number);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_subscription_uuid ON invoices(subscription_uuid);
CREATE INDEX idx_invoices_created_at ON invoices(created_at);

-- 创建更新时间触发器
CREATE TRIGGER update_invoices_updated_at BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

