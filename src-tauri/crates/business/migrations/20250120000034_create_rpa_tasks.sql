-- 创建 rpa_tasks 表
-- RPA 任务表

CREATE TABLE IF NOT EXISTS rpa_tasks (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 基础信息
    name VARCHAR(255) NOT NULL,
    description TEXT,
    tags JSONB DEFAULT '[]',
    -- 触发器: manual, scheduled, event
    trigger_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    -- 调度: hourly, daily, weekly, custom
    schedule VARCHAR(50),
    cron_expression VARCHAR(100),
    -- 运行模式: sequential, parallel
    run_mode VARCHAR(50) NOT NULL DEFAULT 'sequential',
    -- 重试设置
    retry_count INT DEFAULT 0,
    retry_interval INT DEFAULT 5,
    -- 超时（秒）
    timeout INT DEFAULT 300,
    -- 并发数
    concurrency INT DEFAULT 1,
    -- 错误时停止
    stop_on_error BOOLEAN DEFAULT TRUE,
    -- 通知设置
    notify_on_complete BOOLEAN DEFAULT FALSE,
    notify_on_error BOOLEAN DEFAULT TRUE,
    -- 状态: idle, running, completed, failed, cancelled
    status VARCHAR(50) NOT NULL DEFAULT 'idle',
    -- 统计
    run_count INT DEFAULT 0,
    success_count INT DEFAULT 0,
    last_run_at TIMESTAMP WITH TIME ZONE,
    next_run_at TIMESTAMP WITH TIME ZONE,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    -- 约束
    CONSTRAINT fk_rpa_tasks_user FOREIGN KEY (user_uuid) REFERENCES users(uuid),
    CONSTRAINT fk_rpa_tasks_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid)
);

-- 创建索引
CREATE INDEX idx_rpa_tasks_user_uuid ON rpa_tasks(user_uuid);
CREATE INDEX idx_rpa_tasks_team_uuid ON rpa_tasks(team_uuid);
CREATE INDEX idx_rpa_tasks_trigger_type ON rpa_tasks(trigger_type);
CREATE INDEX idx_rpa_tasks_status ON rpa_tasks(status);
CREATE INDEX idx_rpa_tasks_next_run_at ON rpa_tasks(next_run_at);
CREATE INDEX idx_rpa_tasks_deleted_at ON rpa_tasks(deleted_at);

-- 创建更新时间触发器
CREATE TRIGGER update_rpa_tasks_updated_at BEFORE UPDATE ON rpa_tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

