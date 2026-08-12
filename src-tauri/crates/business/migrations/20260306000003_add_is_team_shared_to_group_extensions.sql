-- 为 group_extensions 表添加 is_team_shared 字段
-- 用于区分"分组团队插件"和"分组个人插件"

-- 添加 is_team_shared 字段
ALTER TABLE group_extensions ADD COLUMN is_team_shared BOOLEAN NOT NULL DEFAULT false;

-- 添加索引
CREATE INDEX idx_group_extensions_is_team_shared ON group_extensions(is_team_shared);

-- 更新注释
COMMENT ON COLUMN group_extensions.is_team_shared IS '是否为团队共享: true=团队所有成员可用, false=仅创建者可用';
