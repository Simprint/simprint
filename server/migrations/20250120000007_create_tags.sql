-- 创建 tags 表
-- 标签表

CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(50) DEFAULT 'gray',
    sort_order INT DEFAULT 0,
    -- 统计字段（计算字段，定期更新或触发器维护）
    environments_count INT DEFAULT 0,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_tags_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_tags_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid)
);

-- 创建索引
CREATE INDEX idx_tags_user_uuid ON tags(user_uuid);
CREATE INDEX idx_tags_team_uuid ON tags(team_uuid);
CREATE INDEX idx_tags_deleted_at ON tags(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

