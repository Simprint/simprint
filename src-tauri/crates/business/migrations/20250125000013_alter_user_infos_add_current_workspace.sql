-- 修改 user_infos 表，添加当前工作空间字段
-- 用于工作空间切换功能

ALTER TABLE user_infos
ADD COLUMN current_workspace_uuid TEXT REFERENCES workspaces(uuid) ON DELETE SET NULL;

-- 添加外键约束（如果不存在）

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_infos_current_workspace ON user_infos(current_workspace_uuid);

-- 从当前团队的工作空间初始化 current_workspace_uuid
UPDATE user_infos
SET current_workspace_uuid = (
    SELECT t.workspace_uuid FROM teams t
    WHERE t.uuid = user_infos.current_team_uuid
      AND t.deleted_at IS NULL
    LIMIT 1
)
WHERE current_team_uuid IS NOT NULL
  AND current_workspace_uuid IS NULL;

-- 对于没有当前团队的用户，使用其个人工作空间
UPDATE user_infos
SET current_workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.owner_uuid = user_infos.user_uuid
      AND w.workspace_type = 'personal'
      AND w.deleted_at IS NULL
    LIMIT 1
)
WHERE current_workspace_uuid IS NULL;
