-- Add is_auto_download flag to version_types to control participation in auto-update flows

ALTER TABLE public.version_types
ADD COLUMN IF NOT EXISTS is_auto_download bool DEFAULT true NOT NULL;

COMMENT ON COLUMN public.version_types.is_auto_download IS '是否参与自动检查更新与自动下载';

