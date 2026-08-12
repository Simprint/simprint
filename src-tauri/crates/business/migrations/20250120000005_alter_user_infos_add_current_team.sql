-- 修改 user_infos 表，添加当前团队字段
-- 用于团队切换功能

ALTER TABLE user_infos 
ADD COLUMN IF NOT EXISTS current_team_uuid UUID;

-- 添加外键约束（如果不存在）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_user_infos_current_team'
    ) THEN
        ALTER TABLE user_infos 
        ADD CONSTRAINT fk_user_infos_current_team 
            FOREIGN KEY (current_team_uuid) REFERENCES teams(uuid) ON DELETE SET NULL;
    END IF;
END $$;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_infos_current_team ON user_infos(current_team_uuid);

