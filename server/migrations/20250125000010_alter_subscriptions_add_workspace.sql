-- 修改 subscriptions 表，添加工作空间支持
-- 添加 workspace_uuid，保留 user_uuid（用于记录订阅者）

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_subscriptions_workspace_uuid ON subscriptions(workspace_uuid);

-- 注意：
-- 1. workspace_uuid 的外键约束和数据填充将在数据迁移脚本中完成
-- 2. 唯一约束（确保一个工作空间只有一个活跃订阅）将在数据迁移后添加

