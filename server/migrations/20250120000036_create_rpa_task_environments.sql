-- 创建 rpa_task_environments 表
-- RPA 任务-环境关联表（多对多）

CREATE TABLE IF NOT EXISTS rpa_task_environments (
    id SERIAL PRIMARY KEY,
    task_uuid UUID NOT NULL,
    environment_uuid UUID NOT NULL,
    sort_order INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_rpa_task_env_task FOREIGN KEY (task_uuid) REFERENCES rpa_tasks(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_rpa_task_env_env FOREIGN KEY (environment_uuid) REFERENCES environments(uuid) ON DELETE CASCADE,
    -- 唯一约束
    CONSTRAINT uk_rpa_task_environments UNIQUE (task_uuid, environment_uuid)
);

-- 创建索引
CREATE INDEX idx_rpa_task_env_task_uuid ON rpa_task_environments(task_uuid);
CREATE INDEX idx_rpa_task_env_env_uuid ON rpa_task_environments(environment_uuid);

