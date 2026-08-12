-- Extend versions table with kernel-related metadata fields.
-- These fields are optional for普通资源，但对浏览器内核版本会被要求填充。

ALTER TABLE public.versions
ADD COLUMN IF NOT EXISTS arch text NULL,
ADD COLUMN IF NOT EXISTS package_format text NULL,
ADD COLUMN IF NOT EXISTS requires_extract bool DEFAULT false NOT NULL,
ADD COLUMN IF NOT EXISTS entrypoint_template text NULL,
ADD COLUMN IF NOT EXISTS extract_root text NULL;

COMMENT ON COLUMN public.versions.arch IS '架构（如 x86_64、arm64）';
COMMENT ON COLUMN public.versions.package_format IS '包格式（如 zip、exe）';
COMMENT ON COLUMN public.versions.requires_extract IS '是否需要解压（zip 等资源为 true）';
COMMENT ON COLUMN public.versions.entrypoint_template IS '入口相对路径模板（如 Simprint-Browser/chrome-{version}/chrome.exe）';
COMMENT ON COLUMN public.versions.extract_root IS '可选的解压根目录描述';

