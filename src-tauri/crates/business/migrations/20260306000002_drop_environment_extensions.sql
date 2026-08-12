-- 删除 environment_extensions 表
-- 环境不直接绑定插件，插件通过用户/团队/分组间接作用于环境

-- 删除触发器
DROP TRIGGER IF EXISTS update_environment_extensions_updated_at ON environment_extensions;

-- 删除索引
DROP INDEX IF EXISTS idx_env_extensions_extension_id;
DROP INDEX IF EXISTS idx_env_extensions_env_uuid;

-- 删除表
DROP TABLE IF EXISTS environment_extensions;
