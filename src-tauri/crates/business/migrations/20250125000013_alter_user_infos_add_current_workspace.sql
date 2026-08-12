-- 修改 user_infos 表，添加当前工作空间字段
-- 用于工作空间切换功能

ALTER TABLE user_infos 
ADD COLUMN IF NOT EXISTS current_workspace_uuid UUID;

-- 添加外键约束（如果不存在）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_user_infos_current_workspace'
    ) THEN
        ALTER TABLE user_infos 
        ADD CONSTRAINT fk_user_infos_current_workspace 
            FOREIGN KEY (current_workspace_uuid) REFERENCES workspaces(uuid) ON DELETE SET NULL;
    END IF;
END $$;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_infos_current_workspace ON user_infos(current_workspace_uuid);

-- 从当前团队的工作空间初始化 current_workspace_uuid
UPDATE user_infos ui
SET current_workspace_uuid = (
    SELECT t.workspace_uuid FROM teams t
    WHERE t.uuid = ui.current_team_uuid
      AND t.deleted_at IS NULL
    LIMIT 1
)
WHERE ui.current_team_uuid IS NOT NULL
  AND ui.current_workspace_uuid IS NULL;

-- 对于没有当前团队的用户，使用其个人工作空间
UPDATE user_infos ui
SET current_workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.owner_uuid = ui.user_uuid
      AND w.workspace_type = 'personal'
      AND w.deleted_at IS NULL
    LIMIT 1
)
WHERE ui.current_workspace_uuid IS NULL;

