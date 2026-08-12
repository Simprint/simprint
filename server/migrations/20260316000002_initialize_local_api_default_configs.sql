-- 初始化本地 API 功能默认配置
-- 这些配置作为系统级默认值存在，具体用户数据仍在首次使用时按需创建。

INSERT INTO system_configs (config_key, config_value, description)
VALUES
    ('local_api_default_enabled', 'false', '本地 API 默认是否启用'),
    ('local_api_default_port', '8080', '本地 API 默认监听端口'),
    ('local_api_default_remote_access', 'false', '本地 API 默认是否允许局域网访问'),
    ('local_api_default_cors_origins', '[]', '本地 API 默认允许的 CORS 来源'),
    ('local_api_default_daily_limit', '1000', '本地 API 默认每日总调用上限'),
    ('local_api_default_rate_limit_per_minute', '60', '本地 API 默认每分钟限流'),
    ('local_api_default_rate_limit_per_hour', '1000', '本地 API 默认每小时限流')
ON CONFLICT (config_key) DO NOTHING;
