-- 创建 messages 表
-- 消息表，用于存储系统消息、团队邀请通知等

CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    
    -- 消息类型
    message_type VARCHAR(50) NOT NULL, 
    -- 可选值：
    -- 'private_chat' - 用户私信
    -- 'team_announcement' - 团队公告
    -- 'team_invitation' - 团队邀请
    -- 'team_removal' - 团队移除成员
    -- 'system_notification' - 系统通知
    
    -- 消息内容
    title VARCHAR(255) NOT NULL, -- 消息标题
    content TEXT, -- 消息内容（支持 JSON 格式存储扩展数据）
    
    -- 发送者
    sender_uuid UUID, -- 发送者 UUID（系统消息可为 NULL）
    
    -- 接收者模式
    recipient_type VARCHAR(20) NOT NULL DEFAULT 'single',
    -- 'single' - 单个用户
    -- 'multiple' - 多个指定用户
    -- 'team' - 团队内所有成员
    -- 'all' - 所有用户（系统广播）
    
    -- 关联资源（根据消息类型关联不同的资源）
    related_type VARCHAR(50), -- 关联类型：team, invitation, etc.
    related_uuid UUID, -- 关联资源的 UUID（如 team_uuid, invitation_uuid）
    
    -- 消息元数据（JSON 格式，存储扩展信息）
    metadata JSONB, 
    -- 例如：{
    --   "team_name": "研发团队",
    --   "inviter_name": "张三",
    --   "role": "editor"
    -- }
    
    -- 消息状态
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, deleted
    priority VARCHAR(20) NOT NULL DEFAULT 'normal', -- low, normal, high, urgent
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- 外键约束
    CONSTRAINT fk_messages_sender FOREIGN KEY (sender_uuid) 
        REFERENCES users(uuid) ON DELETE SET NULL
);

-- 创建 user_messages 表（用户消息关联表，支持多接收者）
CREATE TABLE IF NOT EXISTS user_messages (
    id SERIAL PRIMARY KEY,
    message_uuid UUID NOT NULL,
    user_uuid UUID NOT NULL,
    
    -- 阅读状态
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMP WITH TIME ZONE,
    
    -- 操作状态（用于邀请、移除等需要操作的消息）
    action_status VARCHAR(20), 
    -- 'pending' - 待处理（邀请类消息）
    -- 'accepted' - 已接受
    -- 'rejected' - 已拒绝
    -- 'expired' - 已过期
    -- NULL - 无需操作的消息
    
    action_at TIMESTAMP WITH TIME ZONE,
    
    -- 时间戳
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 外键约束
    CONSTRAINT fk_user_messages_message FOREIGN KEY (message_uuid) 
        REFERENCES messages(uuid) ON DELETE CASCADE,
    CONSTRAINT fk_user_messages_user FOREIGN KEY (user_uuid) 
        REFERENCES users(uuid) ON DELETE CASCADE,
    
    -- 唯一约束：一个用户对一条消息只能有一条记录
    CONSTRAINT uk_user_messages_message_user UNIQUE (message_uuid, user_uuid)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_messages_type ON messages(message_type);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_uuid);
CREATE INDEX IF NOT EXISTS idx_messages_related ON messages(related_type, related_uuid);
CREATE INDEX IF NOT EXISTS idx_messages_recipient_type ON messages(recipient_type);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status);
CREATE INDEX IF NOT EXISTS idx_messages_priority ON messages(priority);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_deleted_at ON messages(deleted_at);

CREATE INDEX IF NOT EXISTS idx_user_messages_user ON user_messages(user_uuid);
CREATE INDEX IF NOT EXISTS idx_user_messages_message ON user_messages(message_uuid);
CREATE INDEX IF NOT EXISTS idx_user_messages_is_read ON user_messages(user_uuid, is_read);
CREATE INDEX IF NOT EXISTS idx_user_messages_action_status ON user_messages(user_uuid, action_status);
CREATE INDEX IF NOT EXISTS idx_user_messages_user_unread ON user_messages(user_uuid, is_read) 
    WHERE is_read = FALSE;

-- 复合索引：用于查询用户未读消息
CREATE INDEX IF NOT EXISTS idx_user_messages_user_type_unread 
    ON user_messages(user_uuid, is_read, created_at DESC) 
    WHERE is_read = FALSE;

-- 创建更新时间触发器
CREATE TRIGGER update_messages_updated_at BEFORE UPDATE ON messages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_messages_updated_at BEFORE UPDATE ON user_messages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 团队消息自动分发触发器
-- 当创建 recipient_type='team' 的消息时，自动为团队成员创建 user_messages 记录
CREATE OR REPLACE FUNCTION auto_create_team_message_recipients()
RETURNS TRIGGER AS $$
BEGIN
    -- 如果接收者类型是 team，且有关联的团队 UUID
    IF NEW.recipient_type = 'team' AND NEW.related_type = 'team' AND NEW.related_uuid IS NOT NULL THEN
        -- 为团队所有活跃成员创建消息关联记录
        INSERT INTO user_messages (message_uuid, user_uuid, is_read, action_status)
        SELECT NEW.uuid, tm.user_uuid, FALSE, 
               CASE 
                   WHEN NEW.message_type = 'team_invitation' THEN 'pending'
                   ELSE NULL
               END
        FROM team_members tm
        WHERE tm.team_uuid = NEW.related_uuid
          AND tm.status = 'active'
          AND tm.deleted_at IS NULL
        ON CONFLICT (message_uuid, user_uuid) DO NOTHING;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_auto_create_team_message_recipients
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION auto_create_team_message_recipients();

