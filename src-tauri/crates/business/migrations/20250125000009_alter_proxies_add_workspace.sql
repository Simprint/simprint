-- 修改 proxies 表，添加工作空间支持
-- 添加 workspace_uuid 和 owner_uuid（重命名自 user_uuid），移除 team_uuid, usage_count

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE proxies ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 添加 owner_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE proxies ADD COLUMN IF NOT EXISTS owner_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_proxies_workspace_uuid ON proxies(workspace_uuid);
CREATE INDEX IF NOT EXISTS idx_proxies_owner_uuid ON proxies(owner_uuid);

-- 将 user_uuid 的数据复制到 owner_uuid（如果 owner_uuid 为空）
UPDATE proxies SET owner_uuid = user_uuid WHERE owner_uuid IS NULL;

-- 移除 team_uuid（代理属于工作空间，不属于团队）
ALTER TABLE proxies DROP COLUMN IF EXISTS team_uuid;

-- 移除 usage_count（可通过查询计算）
ALTER TABLE proxies DROP COLUMN IF EXISTS usage_count;

-- 删除旧的索引
DROP INDEX IF EXISTS idx_proxies_team_uuid;

-- 注意：
-- 1. workspace_uuid 和 owner_uuid 的外键约束将在数据迁移脚本中完成
-- 2. user_uuid 列将在数据迁移后删除（迁移到 owner_uuid）

