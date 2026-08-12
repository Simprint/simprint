-- 为 extensions 表添加 hash 字段
-- 用于存储扩展文件的哈希值

ALTER TABLE extensions
ADD COLUMN hash VARCHAR(255) DEFAULT NULL;

-- 为 hash 字段创建索引（可选，如果需要通过 hash 查询）
CREATE INDEX idx_extensions_hash ON extensions(hash);
