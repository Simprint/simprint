-- 创建 environments 表
-- 环境基础信息表

CREATE TABLE IF NOT EXISTS environments (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50) DEFAULT 'chrome',
    icon_color VARCHAR(50) DEFAULT 'text-gray-500',
    -- 状态: ready, running, error
    status VARCHAR(50) NOT NULL DEFAULT 'ready',
    -- 【关联】分组
    group_uuid UUID,
    -- 【关联】代理（环境直接使用的代理，优先级高于分组默认代理）
    proxy_uuid UUID,
    -- 摘要信息（用于列表显示）
    system_info VARCHAR(100),
    kernel_info VARCHAR(100),
    fingerprint_summary VARCHAR(255),
    -- 时间
    last_opened_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_environments_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_environments_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid),
    CONSTRAINT fk_environments_group FOREIGN KEY (group_uuid) REFERENCES groups(uuid) ON DELETE SET NULL,
    CONSTRAINT fk_environments_proxy FOREIGN KEY (proxy_uuid) REFERENCES proxies(uuid) ON DELETE SET NULL
);

-- 创建索引
CREATE INDEX idx_environments_user_uuid ON environments(user_uuid);
CREATE INDEX idx_environments_team_uuid ON environments(team_uuid);
CREATE INDEX idx_environments_group_uuid ON environments(group_uuid);
CREATE INDEX idx_environments_proxy_uuid ON environments(proxy_uuid);
CREATE INDEX idx_environments_status ON environments(status);
CREATE INDEX idx_environments_deleted_at ON environments(deleted_at);
CREATE INDEX idx_environments_name ON environments(name);

-- 创建更新时间触发器
CREATE TRIGGER update_environments_updated_at BEFORE UPDATE ON environments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

