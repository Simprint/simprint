-- 创建 rpa_task_steps 表
-- RPA 任务步骤表

CREATE TABLE IF NOT EXISTS rpa_task_steps (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    task_uuid UUID NOT NULL,
    -- 步骤类型: navigate, click, input, wait, screenshot, script, condition, loop, scroll, keyboard, download, upload
    step_type VARCHAR(50) NOT NULL,
    -- 步骤名称
    name VARCHAR(255) NOT NULL,
    -- 步骤配置（JSON）
    config JSONB NOT NULL DEFAULT '{}',
    -- 是否启用
    enabled BOOLEAN DEFAULT TRUE,
    -- 画布位置
    position_x INT DEFAULT 0,
    position_y INT DEFAULT 0,
    -- 排序
    sort_order INT DEFAULT 0,
    -- 连接到的下一个步骤
    next_step_uuid UUID,
    -- 条件分支（条件类型步骤）
    branch_config JSONB,
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_rpa_task_steps_task FOREIGN KEY (task_uuid) REFERENCES rpa_tasks(uuid) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_rpa_task_steps_task_uuid ON rpa_task_steps(task_uuid);
CREATE INDEX idx_rpa_task_steps_sort_order ON rpa_task_steps(sort_order);

-- 创建更新时间触发器
CREATE TRIGGER update_rpa_task_steps_updated_at BEFORE UPDATE ON rpa_task_steps
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

