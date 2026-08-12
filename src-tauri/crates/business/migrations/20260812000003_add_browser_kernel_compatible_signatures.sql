ALTER TABLE browser_kernel_artifacts
    ADD COLUMN compatible_executable_signatures TEXT NOT NULL DEFAULT '[]';

ALTER TABLE browser_kernel_artifacts
    ADD COLUMN install_dir_name TEXT;

UPDATE browser_kernel_artifacts
SET install_dir_name = resource_name
WHERE install_dir_name IS NULL;
