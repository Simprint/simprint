-- 添加延迟创建的外键约束
-- 某些外键需要在相关表都创建后才能添加

-- teams.default_proxy_uuid -> proxies.uuid

-- groups.default_proxy_uuid -> proxies.uuid

-- 创建 groups.default_proxy_uuid 索引
