-- Console Gateway 管理员表
-- 用于存储 console-gateway 的管理员信息

CREATE TABLE IF NOT EXISTS console_admins (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_console_admins_user_uuid ON console_admins(user_uuid);
CREATE INDEX IF NOT EXISTS idx_console_admins_is_active ON console_admins(is_active);
CREATE INDEX IF NOT EXISTS idx_console_admins_deleted_at ON console_admins(deleted_at);

-- 注释
COMMENT ON TABLE console_admins IS 'Console Gateway 管理员表';
COMMENT ON COLUMN console_admins.id IS '主键 ID';
COMMENT ON COLUMN console_admins.uuid IS '唯一标识';
COMMENT ON COLUMN console_admins.user_uuid IS '关联的用户 UUID';
COMMENT ON COLUMN console_admins.is_active IS '是否激活';
COMMENT ON COLUMN console_admins.created_at IS '创建时间';
COMMENT ON COLUMN console_admins.updated_at IS '更新时间';
COMMENT ON COLUMN console_admins.deleted_at IS '删除时间 (软删除)';

