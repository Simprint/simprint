-- 数据迁移脚本：将现有数据迁移到工作空间架构
-- 此脚本将：
-- 1. 为每个用户创建默认工作空间（personal 类型）
-- 2. 为每个现有团队创建对应的工作空间（team 类型）
-- 3. 迁移配额数据从 user_quotas 到 workspace_quotas
-- 4. 更新所有相关表的外键关系

BEGIN;

-- ============================================
-- 步骤 1: 为每个用户创建默认工作空间
-- ============================================
INSERT INTO workspaces (uuid, name, owner_uuid, workspace_type, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    COALESCE(ui.nickname, ui.email, '我的工作空间') || ' 的工作空间',
    u.uuid,
    'personal',
    u.created_at,
    u.updated_at
FROM users u
LEFT JOIN user_infos ui ON u.uuid = ui.user_uuid
WHERE u.deleted_at IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM workspaces w 
    WHERE w.owner_uuid = u.uuid 
      AND w.workspace_type = 'personal'
      AND w.deleted_at IS NULL
  );

-- ============================================
-- 步骤 2: 为每个现有团队创建对应的工作空间
-- ============================================
INSERT INTO workspaces (uuid, name, owner_uuid, workspace_type, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    t.name || ' 的工作空间',
    t.owner_uuid,
    'team',
    t.created_at,
    t.updated_at
FROM teams t
WHERE t.deleted_at IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM workspaces w 
    WHERE w.owner_uuid = t.owner_uuid 
      AND w.name = t.name || ' 的工作空间'
      AND w.deleted_at IS NULL
  );

-- ============================================
-- 步骤 3: 更新 teams 表的 workspace_uuid
-- ============================================
-- 为每个团队分配工作空间（优先使用团队对应的工作空间，如果没有则使用团队所有者的个人工作空间）
UPDATE teams t
SET workspace_uuid = COALESCE(
    (SELECT w.uuid FROM workspaces w 
     WHERE w.owner_uuid = t.owner_uuid 
       AND w.workspace_type = 'team' 
       AND w.name = t.name || ' 的工作空间'
       AND w.deleted_at IS NULL
     LIMIT 1),
    (SELECT w.uuid FROM workspaces w 
     WHERE w.owner_uuid = t.owner_uuid 
       AND w.workspace_type = 'personal'
       AND w.deleted_at IS NULL
     LIMIT 1)
)
WHERE t.workspace_uuid IS NULL
  AND t.deleted_at IS NULL;

-- 设置 workspace_uuid 为 NOT NULL
ALTER TABLE teams ALTER COLUMN workspace_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE teams 
ADD CONSTRAINT fk_teams_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

-- ============================================
-- 步骤 4: 更新 team_members 表的 workspace_uuid
-- ============================================
UPDATE team_members tm
SET workspace_uuid = t.workspace_uuid
FROM teams t
WHERE tm.team_uuid = t.uuid
  AND tm.workspace_uuid IS NULL
  AND tm.deleted_at IS NULL;

-- 设置 workspace_uuid 为 NOT NULL
ALTER TABLE team_members ALTER COLUMN workspace_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE team_members 
ADD CONSTRAINT fk_team_members_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

-- ============================================
-- 步骤 5: 更新 groups 表的 workspace_uuid
-- ============================================
-- 首先更新有 team_uuid 的分组
UPDATE groups g
SET workspace_uuid = t.workspace_uuid
FROM teams t
WHERE g.team_uuid = t.uuid
  AND g.workspace_uuid IS NULL
  AND g.deleted_at IS NULL;

-- 对于没有 team_uuid 的分组，先分配一个默认团队
UPDATE groups g
SET team_uuid = (
    SELECT t.uuid FROM teams t
    INNER JOIN workspaces w ON t.workspace_uuid = w.uuid
    WHERE w.owner_uuid = (
        -- 尝试从 environments 表找到关联的用户
        SELECT e.user_uuid FROM environments e
        WHERE e.group_uuid = g.uuid
          AND e.deleted_at IS NULL
        LIMIT 1
    )
    AND t.deleted_at IS NULL
    LIMIT 1
)
WHERE g.team_uuid IS NULL
  AND g.deleted_at IS NULL;

-- 再次更新 workspace_uuid（包括刚刚分配了 team_uuid 的分组）
UPDATE groups g
SET workspace_uuid = t.workspace_uuid
FROM teams t
WHERE g.team_uuid = t.uuid
  AND g.workspace_uuid IS NULL
  AND g.deleted_at IS NULL;

-- 对于仍然没有 workspace_uuid 的分组，使用用户的个人工作空间
UPDATE groups g
SET workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.owner_uuid = (
        SELECT e.user_uuid FROM environments e
        WHERE e.group_uuid = g.uuid
          AND e.deleted_at IS NULL
        LIMIT 1
    )
    AND w.workspace_type = 'personal'
    AND w.deleted_at IS NULL
    LIMIT 1
)
WHERE g.workspace_uuid IS NULL
  AND g.deleted_at IS NULL;

-- 如果还有 NULL 值，使用第一个可用的工作空间（兜底方案）
UPDATE groups g
SET workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.deleted_at IS NULL
    ORDER BY w.created_at
    LIMIT 1
)
WHERE g.workspace_uuid IS NULL
  AND g.deleted_at IS NULL;

-- 验证：确保所有分组都有 workspace_uuid
DO $$
DECLARE
    null_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO null_count
    FROM groups
    WHERE workspace_uuid IS NULL
      AND deleted_at IS NULL;
    
    IF null_count > 0 THEN
        RAISE EXCEPTION '仍有 % 个分组的 workspace_uuid 为 NULL，请手动处理', null_count;
    END IF;
END $$;

-- 设置 workspace_uuid 为 NOT NULL
ALTER TABLE groups ALTER COLUMN workspace_uuid SET NOT NULL;

-- 确保 team_uuid 为 NOT NULL（删除没有 team_uuid 的分组或分配默认团队）
-- 注意：这里假设所有分组都应该有 team_uuid，如果没有则可能需要手动处理
UPDATE groups g
SET team_uuid = (
    SELECT t.uuid FROM teams t
    INNER JOIN workspaces w ON t.workspace_uuid = w.uuid
    WHERE w.owner_uuid = (
        SELECT owner_uuid FROM workspaces WHERE uuid = g.workspace_uuid
    )
    AND t.deleted_at IS NULL
    LIMIT 1
)
WHERE g.team_uuid IS NULL
AND g.deleted_at IS NULL;

-- 设置 team_uuid 为 NOT NULL（如果还有 NULL 值，可能需要手动处理）
-- ALTER TABLE groups ALTER COLUMN team_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE groups 
ADD CONSTRAINT fk_groups_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

-- ============================================
-- 步骤 6: 更新 environments 表的 workspace_uuid
-- ============================================
UPDATE environments e
SET workspace_uuid = t.workspace_uuid
FROM teams t
WHERE e.team_uuid = t.uuid
  AND e.workspace_uuid IS NULL
  AND e.deleted_at IS NULL;

-- 对于没有 team_uuid 的环境，使用用户的个人工作空间
UPDATE environments e
SET workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.owner_uuid = e.user_uuid
      AND w.workspace_type = 'personal'
      AND w.deleted_at IS NULL
    LIMIT 1
)
WHERE e.workspace_uuid IS NULL
  AND e.deleted_at IS NULL;

-- 设置 workspace_uuid 为 NOT NULL
ALTER TABLE environments ALTER COLUMN workspace_uuid SET NOT NULL;

-- 确保 team_uuid 为 NOT NULL（为没有 team_uuid 的环境分配默认团队）
UPDATE environments e
SET team_uuid = (
    SELECT t.uuid FROM teams t
    INNER JOIN workspaces w ON t.workspace_uuid = w.uuid
    WHERE w.uuid = e.workspace_uuid
      AND t.deleted_at IS NULL
    LIMIT 1
)
WHERE e.team_uuid IS NULL
  AND e.deleted_at IS NULL;

-- 设置 team_uuid 为 NOT NULL（如果还有 NULL 值，可能需要手动处理）
-- ALTER TABLE environments ALTER COLUMN team_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE environments 
ADD CONSTRAINT fk_environments_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

-- ============================================
-- 步骤 7: 更新 proxies 表的 workspace_uuid 和 owner_uuid
-- ============================================
-- 确保 owner_uuid 已填充（从 user_uuid 复制）
UPDATE proxies p
SET owner_uuid = p.user_uuid
WHERE p.owner_uuid IS NULL
  AND p.deleted_at IS NULL;

-- 为代理分配工作空间（优先使用团队的工作空间，如果没有则使用用户的个人工作空间）
UPDATE proxies p
SET workspace_uuid = COALESCE(
    (SELECT t.workspace_uuid FROM teams t
     WHERE t.uuid = (
         SELECT e.team_uuid FROM environments e
         WHERE e.proxy_uuid = p.uuid
           AND e.deleted_at IS NULL
         LIMIT 1
     )
     AND t.deleted_at IS NULL
     LIMIT 1),
    (SELECT w.uuid FROM workspaces w
     WHERE w.owner_uuid = p.owner_uuid
       AND w.workspace_type = 'personal'
       AND w.deleted_at IS NULL
     LIMIT 1)
)
WHERE p.workspace_uuid IS NULL
  AND p.deleted_at IS NULL;

-- 设置 workspace_uuid 和 owner_uuid 为 NOT NULL
ALTER TABLE proxies ALTER COLUMN workspace_uuid SET NOT NULL;
ALTER TABLE proxies ALTER COLUMN owner_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE proxies 
ADD CONSTRAINT fk_proxies_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

ALTER TABLE proxies 
ADD CONSTRAINT fk_proxies_owner 
FOREIGN KEY (owner_uuid) REFERENCES users(uuid) ON DELETE CASCADE;

-- 删除旧的 user_uuid 列（如果存在）
ALTER TABLE proxies DROP COLUMN IF EXISTS user_uuid;

-- ============================================
-- 步骤 8: 更新 subscriptions 表的 workspace_uuid
-- ============================================
-- 为订阅分配工作空间（使用订阅者的个人工作空间）
UPDATE subscriptions s
SET workspace_uuid = (
    SELECT w.uuid FROM workspaces w
    WHERE w.owner_uuid = s.user_uuid
      AND w.workspace_type = 'personal'
      AND w.deleted_at IS NULL
    LIMIT 1
)
WHERE s.workspace_uuid IS NULL;

-- 设置 workspace_uuid 为 NOT NULL
ALTER TABLE subscriptions ALTER COLUMN workspace_uuid SET NOT NULL;

-- 添加外键约束
ALTER TABLE subscriptions 
ADD CONSTRAINT fk_subscriptions_workspace 
FOREIGN KEY (workspace_uuid) REFERENCES workspaces(uuid) ON DELETE CASCADE;

-- ============================================
-- 步骤 9: 迁移配额数据从 user_quotas 到 workspace_quotas
-- ============================================
INSERT INTO workspace_quotas (
    workspace_uuid,
    max_environments,
    used_environments,
    max_team_members,
    used_team_members,
    max_proxies,
    used_proxies,
    max_rpa_tasks,
    used_rpa_tasks,
    created_at,
    updated_at
)
SELECT 
    w.uuid,
    COALESCE(uq.max_environments, 10),
    COALESCE(uq.used_environments, 0),
    COALESCE(uq.max_team_members, 5),
    0,  -- used_team_members: user_quotas 表中没有此字段，使用默认值 0
    COALESCE(uq.max_proxies, 10),
    COALESCE(uq.used_proxies, 0),
    COALESCE(uq.max_rpa_tasks, 5),
    COALESCE(uq.used_rpa_tasks, 0),
    COALESCE(uq.created_at, CURRENT_TIMESTAMP),
    COALESCE(uq.updated_at, CURRENT_TIMESTAMP)
FROM workspaces w
LEFT JOIN user_quotas uq ON w.owner_uuid = uq.user_uuid
WHERE w.deleted_at IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM workspace_quotas wq 
    WHERE wq.workspace_uuid = w.uuid
  );

-- ============================================
-- 步骤 10: 更新配额使用情况统计
-- ============================================
-- 更新环境使用数
UPDATE workspace_quotas wq
SET used_environments = (
    SELECT COUNT(*) FROM environments e
    WHERE e.workspace_uuid = wq.workspace_uuid
      AND e.deleted_at IS NULL
);

-- 更新代理使用数
UPDATE workspace_quotas wq
SET used_proxies = (
    SELECT COUNT(*) FROM proxies p
    WHERE p.workspace_uuid = wq.workspace_uuid
      AND p.deleted_at IS NULL
);

-- 更新团队成员使用数（所有团队总和）
UPDATE workspace_quotas wq
SET used_team_members = (
    SELECT COUNT(DISTINCT tm.user_uuid) FROM team_members tm
    INNER JOIN teams t ON tm.team_uuid = t.uuid
    WHERE t.workspace_uuid = wq.workspace_uuid
      AND tm.deleted_at IS NULL
      AND t.deleted_at IS NULL
);

-- 更新 RPA 任务使用数（通过 team_uuid 关联到工作空间）
UPDATE workspace_quotas wq
SET used_rpa_tasks = (
    SELECT COUNT(*) FROM rpa_tasks rt
    INNER JOIN teams t ON rt.team_uuid = t.uuid
    WHERE t.workspace_uuid = wq.workspace_uuid
      AND rt.deleted_at IS NULL
      AND t.deleted_at IS NULL
)
WHERE EXISTS (
    SELECT 1 FROM information_schema.tables 
    WHERE table_name = 'rpa_tasks'
);

COMMIT;

-- 迁移完成后的验证查询（可选，用于检查迁移结果）
-- SELECT 
--     'workspaces' as table_name,
--     COUNT(*) as total_count,
--     COUNT(*) FILTER (WHERE deleted_at IS NULL) as active_count
-- FROM workspaces
-- UNION ALL
-- SELECT 
--     'teams',
--     COUNT(*),
--     COUNT(*) FILTER (WHERE deleted_at IS NULL)
-- FROM teams
-- UNION ALL
-- SELECT 
--     'workspace_quotas',
--     COUNT(*),
--     COUNT(*)
-- FROM workspace_quotas;

