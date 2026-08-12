-- 添加延迟创建的外键约束
-- 某些外键需要在相关表都创建后才能添加

-- teams.default_proxy_uuid -> proxies.uuid
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_teams_default_proxy'
    ) THEN
        ALTER TABLE teams 
        ADD CONSTRAINT fk_teams_default_proxy 
            FOREIGN KEY (default_proxy_uuid) REFERENCES proxies(uuid) ON DELETE SET NULL;
    END IF;
END $$;

-- groups.default_proxy_uuid -> proxies.uuid
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_groups_default_proxy'
    ) THEN
        ALTER TABLE groups 
        ADD CONSTRAINT fk_groups_default_proxy 
            FOREIGN KEY (default_proxy_uuid) REFERENCES proxies(uuid) ON DELETE SET NULL;
    END IF;
END $$;

-- 创建 groups.default_proxy_uuid 索引
CREATE INDEX IF NOT EXISTS idx_groups_default_proxy ON groups(default_proxy_uuid);

