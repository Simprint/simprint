-- 修正扩展表的状态默认值和注释
-- 将状态从 "installed, disabled" 改为 "active, inactive" 以对齐代码实现

-- user_extensions 表
ALTER TABLE user_extensions ALTER COLUMN status SET DEFAULT 'active';
COMMENT ON COLUMN user_extensions.status IS '状态: active, inactive';

-- team_extensions 表
ALTER TABLE team_extensions ALTER COLUMN status SET DEFAULT 'active';
COMMENT ON COLUMN team_extensions.status IS '状态: active, inactive';

-- group_extensions 表
ALTER TABLE group_extensions ALTER COLUMN status SET DEFAULT 'active';
COMMENT ON COLUMN group_extensions.status IS '状态: active, inactive';

-- environment_extensions 表
ALTER TABLE environment_extensions ALTER COLUMN status SET DEFAULT 'active';
COMMENT ON COLUMN environment_extensions.status IS '状态: active, inactive';
