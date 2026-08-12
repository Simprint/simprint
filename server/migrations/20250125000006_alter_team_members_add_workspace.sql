-- 修改 team_members 表，添加工作空间支持
-- 添加 workspace_uuid（冗余字段），移除统计字段

-- 添加 workspace_uuid 列（先允许 NULL，数据迁移后再设置为 NOT NULL）
ALTER TABLE team_members ADD COLUMN IF NOT EXISTS workspace_uuid UUID;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_team_members_workspace_uuid ON team_members(workspace_uuid);

-- 移除统计字段（可通过查询计算）
ALTER TABLE team_members DROP COLUMN IF EXISTS environment_count;
ALTER TABLE team_members DROP COLUMN IF EXISTS group_count;

-- 删除旧的唯一约束
ALTER TABLE team_members DROP CONSTRAINT IF EXISTS uk_team_members;

-- 添加新的唯一约束（包含 workspace_uuid）
ALTER TABLE team_members ADD CONSTRAINT uk_team_members UNIQUE (team_uuid, user_uuid, workspace_uuid);

-- 注意：workspace_uuid 的外键约束和数据填充将在数据迁移脚本中完成

