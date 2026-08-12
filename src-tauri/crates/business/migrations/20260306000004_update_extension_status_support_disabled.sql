-- 更新扩展状态支持 disabled
-- 允许用户禁用团队/分组插件

-- 更新状态注释
COMMENT ON COLUMN user_extensions.status IS '状态: active=已启用, disabled=已禁用, inactive=已卸载';
COMMENT ON COLUMN team_extensions.status IS '状态: active=已安装, inactive=已卸载';
COMMENT ON COLUMN group_extensions.status IS '状态: active=已安装, inactive=已卸载';
