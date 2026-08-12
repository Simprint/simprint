-- Local authentication is intentionally separate from the former cloud-account fields.
-- `users` remains the owner boundary for all business data, while this table contains
-- only the information needed by the local user picker.
CREATE TABLE IF NOT EXISTS local_user_auth (
    user_uuid TEXT PRIMARY KEY,
    avatar TEXT NOT NULL,
    password_salt TEXT,
    password_hash TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_local_user_auth_user
        FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE,
    CONSTRAINT chk_local_user_auth_password
        CHECK (
            (password_salt IS NULL AND password_hash IS NULL)
            OR (password_salt IS NOT NULL AND password_hash IS NOT NULL)
        )
);

-- Preserve users created by builds that predate the local user picker. Their legacy
-- account password is deliberately not imported because it used a different scheme.
INSERT INTO local_user_auth (user_uuid, avatar)
SELECT u.uuid, COALESCE(NULLIF(ui.avatar_hash, ''), '🙂')
FROM users u
JOIN user_infos ui ON ui.user_uuid = u.uuid
WHERE u.deleted_at IS NULL AND ui.deleted_at IS NULL
ON CONFLICT (user_uuid) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_local_user_auth_created_at
    ON local_user_auth(created_at);
