-- 创建 audit_logs 表
-- 审计日志表

CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_uuid UUID NOT NULL,
    team_uuid UUID,
    -- 操作类型: login, logout, password_change, create, update, delete, batch_delete, start, stop, import, export, invite, role_change, member_remove, settings_update
    action VARCHAR(50) NOT NULL,
    -- 目标类型: environment, group, tag, proxy, account, team, settings, system
    target_type VARCHAR(50) NOT NULL,
    -- 目标 ID
    target_uuid UUID,
    target_name VARCHAR(255),
    -- 详情
    details TEXT,
    -- 变更内容（JSON）
    changes JSONB,
    -- 请求信息
    ip_address VARCHAR(45),
    user_agent TEXT,
    request_id VARCHAR(100),
    -- 时间
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_audit_logs_user FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);

-- 创建索引
CREATE INDEX idx_audit_logs_user_uuid ON audit_logs(user_uuid);
CREATE INDEX idx_audit_logs_team_uuid ON audit_logs(team_uuid);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_target_type ON audit_logs(target_type);
CREATE INDEX idx_audit_logs_target_uuid ON audit_logs(target_uuid);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

