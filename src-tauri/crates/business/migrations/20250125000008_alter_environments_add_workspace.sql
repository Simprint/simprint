-- 修改 environments 表，添加工作空间支持
-- 添加 workspace_uuid，确保 team_uuid NOT NULL

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE environments ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_environments_workspace_uuid ON environments(workspace_uuid);

-- 注意：
-- 1. workspace_uuid 的外键约束和数据填充将在数据迁移脚本中完成
-- 2. team_uuid 的 NOT NULL 约束将在数据迁移后添加（确保所有数据都有 team_uuid）

