-- 修改 groups 表，添加工作空间支持
-- 添加 workspace_uuid，移除 user_uuid, default_proxy_uuid, color，确保 team_uuid NOT NULL

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE groups ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_groups_workspace_uuid ON groups(workspace_uuid);

-- 移除 user_uuid（分组属于团队，不再直接属于用户）
ALTER TABLE groups DROP COLUMN IF EXISTS user_uuid;

-- 移除默认代理字段（不再需要默认代理）
ALTER TABLE groups DROP COLUMN IF EXISTS default_proxy_uuid;

-- 移除 color 字段（简化设计）
ALTER TABLE groups DROP COLUMN IF EXISTS color;

-- 删除旧的索引
DROP INDEX IF EXISTS idx_groups_user_uuid;

-- 注意：
-- 1. workspace_uuid 的外键约束和数据填充将在数据迁移脚本中完成
-- 2. team_uuid 的 NOT NULL 约束将在数据迁移后添加（确保所有数据都有 team_uuid）

