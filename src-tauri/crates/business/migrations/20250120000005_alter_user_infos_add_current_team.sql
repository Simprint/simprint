-- 修改 user_infos 表，添加当前团队字段
-- 用于团队切换功能

ALTER TABLE user_infos
ADD COLUMN current_team_uuid TEXT REFERENCES teams(uuid) ON DELETE SET NULL;

-- 添加外键约束（如果不存在）

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_infos_current_team ON user_infos(current_team_uuid);
