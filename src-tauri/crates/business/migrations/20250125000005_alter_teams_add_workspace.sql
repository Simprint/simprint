-- 修改 teams 表，添加工作空间支持
-- 添加 workspace_uuid，移除配额相关字段和默认代理

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE teams ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_teams_workspace_uuid ON teams(workspace_uuid);

-- 移除配额相关字段（配额移至 workspace_quotas）
ALTER TABLE teams DROP COLUMN IF EXISTS max_members;
ALTER TABLE teams DROP COLUMN IF EXISTS max_environments;
ALTER TABLE teams DROP COLUMN IF EXISTS max_proxies;

-- 移除默认代理字段（不再需要默认代理）
ALTER TABLE teams DROP COLUMN IF EXISTS default_proxy_uuid;

-- 注意：workspace_uuid 的外键约束和数据填充将在数据迁移脚本中完成

