-- 创建 extensions 表
-- 浏览器扩展表

CREATE TABLE IF NOT EXISTS extensions (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    -- 扩展 ID（Chrome/Firefox 商店 ID）
    extension_id VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    -- 版本信息
    version VARCHAR(50) NOT NULL,
    -- 分类: productivity, privacy, automation, social, development, other
    category VARCHAR(50) NOT NULL DEFAULT 'other',
    -- 浏览器: chrome, firefox, edge, all
    browser VARCHAR(50) NOT NULL DEFAULT 'all',
    -- 开发者信息
    developer VARCHAR(255),
    homepage VARCHAR(512),
    icon_url VARCHAR(512),
    -- 下载信息
    download_url VARCHAR(512),
    file_size BIGINT,
    -- 统计
    downloads_count BIGINT DEFAULT 0,
    rating DECIMAL(3, 2),
    -- 权限
    permissions JSONB DEFAULT '[]',
    -- 状态: active, deprecated, removed
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- 更新日志
    changelog JSONB DEFAULT '[]',
    -- 时间
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_extensions_extension_id ON extensions(extension_id);
CREATE INDEX idx_extensions_category ON extensions(category);
CREATE INDEX idx_extensions_browser ON extensions(browser);
CREATE INDEX idx_extensions_status ON extensions(status);

-- 创建更新时间触发器
CREATE TRIGGER update_extensions_updated_at BEFORE UPDATE ON extensions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

