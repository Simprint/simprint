-- 创建 environment_tags 表
-- 环境-标签关联表（多对多）

CREATE TABLE IF NOT EXISTS environment_tags (
    id SERIAL PRIMARY KEY,
    environment_uuid UUID NOT NULL,
    tag_uuid UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_env_tags_env FOREIGN KEY (environment_uuid) 
        REFERENCES environments(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_env_tags_tag FOREIGN KEY (tag_uuid) 
        REFERENCES tags(uuid) ON DELETE CASCADE,
    -- 唯一约束：同一环境不能重复添加同一标签
    CONSTRAINT uk_env_tags UNIQUE (environment_uuid, tag_uuid)
);

-- 创建索引
CREATE INDEX idx_env_tags_env_uuid ON environment_tags(environment_uuid);
CREATE INDEX idx_env_tags_tag_uuid ON environment_tags(tag_uuid);

