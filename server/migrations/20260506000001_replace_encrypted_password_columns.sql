ALTER TABLE public.proxies
    DROP COLUMN IF EXISTS password_encrypted,
    ADD COLUMN password TEXT;

COMMENT ON COLUMN public.proxies.password IS '代理密码（明文存储）';

ALTER TABLE public.platform_accounts
    DROP COLUMN IF EXISTS password_encrypted,
    ADD COLUMN password TEXT;

COMMENT ON COLUMN public.platform_accounts.password IS '平台账号密码（明文存储）';
