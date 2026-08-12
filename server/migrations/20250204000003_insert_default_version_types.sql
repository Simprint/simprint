-- 初始化版本类型默认数据
-- 注意：使用 ON CONFLICT 以保证迁移可重复执行

INSERT INTO public.version_types (type_code, type_name, description, sort_order, is_active, is_auto_download)
VALUES
  ('SIMPRINT_CLIENT_INSTALLER', 'Simprint 客户端安装包', 'Simprint 浏览器主程序安装包', 10, true, true),
  ('SIMPRINT_KERNEL_CHROMIUM',  'Chromium 内核',        'Chromium 浏览器内核 zip 包', 30, true, false)
ON CONFLICT (type_code) DO NOTHING;