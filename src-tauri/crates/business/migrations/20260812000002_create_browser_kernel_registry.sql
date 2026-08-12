CREATE TABLE IF NOT EXISTS browser_kernel_artifacts (
    kernel_id TEXT PRIMARY KEY,
    type_code TEXT NOT NULL,
    resource_name TEXT NOT NULL,
    version TEXT NOT NULL,
    name TEXT,
    notes TEXT,
    platform TEXT NOT NULL,
    package_hash TEXT NOT NULL,
    executable_signature TEXT NOT NULL,
    file_size INTEGER,
    is_latest INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    arch TEXT NOT NULL,
    package_format TEXT NOT NULL,
    requires_extract INTEGER NOT NULL DEFAULT 0,
    entrypoint_template TEXT,
    extract_root TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_browser_kernel_artifacts_lookup
    ON browser_kernel_artifacts(type_code, platform, status, is_latest);
CREATE INDEX IF NOT EXISTS idx_browser_kernel_artifacts_resource_name
    ON browser_kernel_artifacts(resource_name);

CREATE TABLE IF NOT EXISTS browser_kernel_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kernel_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    url TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_browser_kernel_sources_artifact
        FOREIGN KEY (kernel_id) REFERENCES browser_kernel_artifacts(kernel_id) ON DELETE CASCADE,
    CONSTRAINT uq_browser_kernel_sources UNIQUE (kernel_id, source_id, url)
);

CREATE INDEX IF NOT EXISTS idx_browser_kernel_sources_resolve
    ON browser_kernel_sources(kernel_id, is_active, priority);

CREATE TABLE IF NOT EXISTS browser_kernel_installations (
    kernel_id TEXT PRIMARY KEY,
    install_path TEXT NOT NULL,
    verified_signature TEXT,
    status TEXT NOT NULL DEFAULT 'ready',
    installed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified_at TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_browser_kernel_installations_artifact
        FOREIGN KEY (kernel_id) REFERENCES browser_kernel_artifacts(kernel_id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS environment_kernel_bindings (
    environment_uuid TEXT PRIMARY KEY,
    kernel_id TEXT NOT NULL,
    bound_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_environment_kernel_bindings_environment
        FOREIGN KEY (environment_uuid) REFERENCES environments(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_environment_kernel_bindings_artifact
        FOREIGN KEY (kernel_id) REFERENCES browser_kernel_artifacts(kernel_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_environment_kernel_bindings_kernel_id
    ON environment_kernel_bindings(kernel_id);
