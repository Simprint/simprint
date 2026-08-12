-- 创建 rpa_task_runs 表
-- RPA 任务执行记录表

CREATE TABLE IF NOT EXISTS rpa_task_runs (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    task_uuid UUID NOT NULL,
    -- 状态: running, completed, failed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    -- 步骤统计
    total_steps INT NOT NULL DEFAULT 0,
    completed_steps INT NOT NULL DEFAULT 0,
    failed_steps INT NOT NULL DEFAULT 0,
    -- 执行时间
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP WITH TIME ZONE,
    duration_ms BIGINT,
    -- 结果摘要
    result_summary TEXT,
    -- 错误信息
    error_message TEXT,
    -- 执行日志（JSON 数组）
    logs JSONB DEFAULT '[]',
    -- 约束
    CONSTRAINT fk_rpa_task_runs_task FOREIGN KEY (task_uuid) REFERENCES rpa_tasks(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_rpa_task_runs_task_uuid ON rpa_task_runs(task_uuid);
CREATE INDEX idx_rpa_task_runs_status ON rpa_task_runs(status);
CREATE INDEX idx_rpa_task_runs_started_at ON rpa_task_runs(started_at);

