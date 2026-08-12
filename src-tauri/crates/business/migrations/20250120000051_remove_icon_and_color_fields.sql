-- 移除环境的 icon 和 icon_color 字段
-- 移除分组的 color 字段

-- 移除 environments 表的 icon 和 icon_color 字段
ALTER TABLE environments DROP COLUMN icon;
ALTER TABLE environments DROP COLUMN icon_color;

-- 移除 groups 表的 color 字段
ALTER TABLE groups DROP COLUMN color;
