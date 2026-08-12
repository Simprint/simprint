-- 创建用户团队插件偏好设置表
CREATE TABLE IF NOT EXISTS user_team_extension_preferences (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    team_uuid UUID NOT NULL,
    extension_id VARCHAR(255) NOT NULL,
    is_disabled BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_uuid, team_uuid, extension_id)
);

-- 添加索引
CREATE INDEX idx_user_team_extension_preferences_user_uuid ON user_team_extension_preferences(user_uuid);
CREATE INDEX idx_user_team_extension_preferences_team_uuid ON user_team_extension_preferences(team_uuid);
CREATE INDEX idx_user_team_extension_preferences_extension_id ON user_team_extension_preferences(extension_id);

-- 添加注释
COMMENT ON TABLE user_team_extension_preferences IS '用户对团队插件的偏好设置（如禁用）';
COMMENT ON COLUMN user_team_extension_preferences.user_uuid IS '用户 UUID';
COMMENT ON COLUMN user_team_extension_preferences.team_uuid IS '团队 UUID';
COMMENT ON COLUMN user_team_extension_preferences.extension_id IS '插件 ID';
COMMENT ON COLUMN user_team_extension_preferences.is_disabled IS '是否禁用（true 表示用户禁用了该团队插件）';
