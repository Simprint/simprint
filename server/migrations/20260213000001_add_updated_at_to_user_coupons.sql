-- 为 user_coupons 表添加 updated_at 字段
-- 修复订阅功能中缺少 updated_at 字段的问题

ALTER TABLE user_coupons
ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- 创建更新时间触发器
CREATE TRIGGER update_user_coupons_updated_at BEFORE UPDATE ON user_coupons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
