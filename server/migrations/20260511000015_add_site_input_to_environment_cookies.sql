ALTER TABLE environment_cookies
ADD COLUMN IF NOT EXISTS site_input TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_env_cookies_env_uuid_site_input
ON environment_cookies(environment_uuid, site_input);
