-- 创建 proxy_visible_teams 表
-- 代理可见团队关联表，控制代理对哪些团队可见

CREATE TABLE IF NOT EXISTS proxy_visible_teams (
    proxy_uuid TEXT NOT NULL,
    workspace_uuid TEXT NOT NULL,
    team_uuid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 约束
    CONSTRAINT fk_proxy_visible_teams_proxy FOREIGN KEY (proxy_uuid) REFERENCES proxies(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_proxy_visible_teams_workspace FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_proxy_visible_teams_team FOREIGN KEY (team_uuid) REFERENCES teams(uuid) ON DELETE CASCADE,
    -- 唯一约束：一个代理对一个团队只能有一条可见性记录
    CONSTRAINT uk_proxy_visible_teams UNIQUE (proxy_uuid, team_uuid)
);

-- 创建索引
CREATE INDEX idx_proxy_visible_teams_proxy_uuid ON proxy_visible_teams(proxy_uuid);
CREATE INDEX idx_proxy_visible_teams_team_uuid ON proxy_visible_teams(team_uuid);
CREATE INDEX idx_proxy_visible_teams_workspace_uuid ON proxy_visible_teams(workspace_uuid);

-- 列注释
