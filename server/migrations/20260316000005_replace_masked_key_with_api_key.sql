-- 使用完整 api_key 替代 masked_key
-- 旧数据无法从 masked_key 还原明文，因此 api_key 先允许为空；
-- 服务端在发现历史记录缺少 api_key 时，会自动轮换生成新 key。

ALTER TABLE user_local_api_keys
    ADD COLUMN IF NOT EXISTS api_key TEXT;

ALTER TABLE user_local_api_keys
    DROP COLUMN IF EXISTS masked_key;
