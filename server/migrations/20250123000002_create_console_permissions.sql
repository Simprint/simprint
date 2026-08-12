-- Console Gateway 权限表
-- 用于存储 console-gateway 的路由权限

CREATE TABLE IF NOT EXISTS console_permissions (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    route_path VARCHAR(512) NOT NULL,
    method VARCHAR(10) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(route_path, method)
);

-- Console Gateway 管理员权限关联表
CREATE TABLE IF NOT EXISTS console_admin_permissions (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    admin_id INTEGER NOT NULL REFERENCES console_admins(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES console_permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    granted_by UUID,
    UNIQUE(admin_id, permission_id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_console_permissions_route ON console_permissions(route_path, method);
CREATE INDEX IF NOT EXISTS idx_console_permissions_deleted_at ON console_permissions(deleted_at);
CREATE INDEX IF NOT EXISTS idx_console_admin_permissions_admin ON console_admin_permissions(admin_id);
CREATE INDEX IF NOT EXISTS idx_console_admin_permissions_permission ON console_admin_permissions(permission_id);

-- 注释
COMMENT ON TABLE console_permissions IS 'Console Gateway 路由权限表';
COMMENT ON COLUMN console_permissions.id IS '主键 ID';
COMMENT ON COLUMN console_permissions.uuid IS '唯一标识';
COMMENT ON COLUMN console_permissions.route_path IS '路由路径';
COMMENT ON COLUMN console_permissions.method IS 'HTTP 方法 (GET, POST, PUT, DELETE, PATCH)';
COMMENT ON COLUMN console_permissions.description IS '权限描述';
COMMENT ON COLUMN console_permissions.created_at IS '创建时间';
COMMENT ON COLUMN console_permissions.updated_at IS '更新时间';
COMMENT ON COLUMN console_permissions.deleted_at IS '删除时间 (软删除)';

COMMENT ON TABLE console_admin_permissions IS 'Console Gateway 管理员权限关联表';
COMMENT ON COLUMN console_admin_permissions.id IS '主键 ID';
COMMENT ON COLUMN console_admin_permissions.uuid IS '唯一标识';
COMMENT ON COLUMN console_admin_permissions.admin_id IS '管理员 ID';
COMMENT ON COLUMN console_admin_permissions.permission_id IS '权限 ID';
COMMENT ON COLUMN console_admin_permissions.granted_at IS '授权时间';
COMMENT ON COLUMN console_admin_permissions.granted_by IS '授权人 UUID';

