-- 初始化默认浏览器内核版本数据
-- 使用 version_types.type_code 定位类型，避免写死 type_id。
-- 使用 NOT EXISTS 保证迁移可重复执行。

WITH kernel_type AS (
    SELECT id
    FROM public.version_types
    WHERE type_code = 'SIMPRINT_KERNEL_CHROMIUM'
)
INSERT INTO public.versions (
    type_id,
    resource_name,
    version,
    name,
    notes,
    platform,
    url,
    hash,
    signature,
    install_path,
    file_size,
    is_latest,
    status,
    pub_date,
    created_at,
    updated_at,
    deleted_at,
    arch,
    package_format,
    requires_extract,
    entrypoint_template,
    extract_root
)
SELECT
    kernel_type.id,
    'Chrome 144',
    '144.0.7559.118.1',
    'simprint-browser-144.0.7559.118.zip',
    '上传浏览器内核版本。',
    'windows',
    'versions/144.0.7559.118.1/simprint-browser-144.0.7559.118.zip',
    'c74ce58537c93e99e4099c94667a21a8e150ac4052c383bc2a095ffdb3b0e075',
    'a864f950c2e77d18ef932f2ad3dbde63039c3b467994daab83081ce6b28c4f82',
    NULL,
    184526645,
    false,
    'active',
    '2026-05-05 17:01:26.746804+08'::timestamptz,
    '2026-05-05 17:01:26.746804+08'::timestamptz,
    NULL,
    NULL,
    'x86_64',
    'zip',
    true,
    NULL,
    NULL
FROM kernel_type
WHERE NOT EXISTS (
    SELECT 1
    FROM public.versions v
    WHERE v.type_id = kernel_type.id
      AND v.resource_name = 'Chrome 144'
      AND v.version = '144.0.7559.118.1'
      AND v.platform = 'windows'
      AND v.arch = 'x86_64'
      AND v.deleted_at IS NULL
);
