-- 为 team_invitations 表添加 deleted_at 字段
-- 用于软删除功能

ALTER TABLE team_invitations 
ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_invitations_deleted_at ON team_invitations(deleted_at) WHERE deleted_at IS NULL;

