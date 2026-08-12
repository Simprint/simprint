-- 废弃 user_quotas 表
-- 保留表结构，但标记为废弃，数据将迁移到 workspace_quotas

-- 添加注释标记为废弃
COMMENT ON TABLE user_quotas IS '已废弃：配额已迁移到 workspace_quotas 表，此表保留仅用于历史数据查询';

-- 注意：表结构保持不变，数据迁移将在数据迁移脚本中完成

