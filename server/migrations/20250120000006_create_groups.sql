-- 创建 groups 表
-- 环境分组表

CREATE TABLE IF NOT EXISTS groups (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(50) DEFAULT 'gray',
    sort_order INT DEFAULT 0,
    -- 【关联】分组默认代理（外键在 proxies 表创建后添加）
    default_proxy_uuid UUID,
    -- 创建者
    created_by UUID,
    -- 统计字段（计算字段）
    environments_count INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_groups_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_groups_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid),
    CONSTRAINT fk_groups_created_by FOREIGN KEY (created_by) REFERENCES users(uuid)
    -- 注意: default_proxy_uuid 的外键需要在 proxies 表创建后添加
);

-- 创建索引
CREATE INDEX idx_groups_user_uuid ON groups(user_uuid);
CREATE INDEX idx_groups_team_uuid ON groups(team_uuid);
CREATE INDEX idx_groups_deleted_at ON groups(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_groups_updated_at BEFORE UPDATE ON groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

