use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{MessageDto, UserMessageDto};

// ============ Messages ============

/// 创建消息
pub async fn create_message(
    pool: &Pool<Postgres>,
    sender_uuid: Option<Uuid>,
    message_type: &str,
    title: &str,
    content: Option<&str>,
    recipient_type: &str,
    related_type: Option<&str>,
    related_uuid: Option<Uuid>,
    priority: &str,
    metadata: Option<serde_json::Value>,
) -> Result<Uuid, Error> {
    let message_uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO messages (message_type, title, content, sender_uuid, recipient_type, related_type, related_uuid, priority, metadata, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')
        RETURNING uuid;
        "#,
    )
    .bind(message_type)
    .bind(title)
    .bind(content)
    .bind(sender_uuid)
    .bind(recipient_type)
    .bind(related_type)
    .bind(related_uuid)
    .bind(priority)
    .bind(metadata)
    .fetch_one(pool)
    .await?;

    Ok(message_uuid)
}

/// 添加消息接收者
pub async fn add_message_recipient(
    pool: &Pool<Postgres>,
    message_uuid: Uuid,
    user_uuid: Uuid,
    action_status: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO user_messages (message_uuid, user_uuid, is_read, action_status)
        VALUES ($1, $2, FALSE, $3)
        ON CONFLICT (message_uuid, user_uuid) DO NOTHING;
        "#,
    )
    .bind(message_uuid)
    .bind(user_uuid)
    .bind(action_status)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量添加消息接收者
pub async fn add_message_recipients(
    pool: &Pool<Postgres>,
    message_uuid: Uuid,
    user_uuids: &[Uuid],
    action_status: Option<&str>,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    for user_uuid in user_uuids {
        sqlx::query(
            r#"
            INSERT INTO user_messages (message_uuid, user_uuid, is_read, action_status)
            VALUES ($1, $2, FALSE, $3)
            ON CONFLICT (message_uuid, user_uuid) DO NOTHING;
            "#,
        )
        .bind(message_uuid)
        .bind(user_uuid)
        .bind(action_status)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// 查询用户消息列表
pub async fn fetch_user_messages(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    offset: i64,
    limit: i64,
    message_type: Option<&str>,
    is_read: Option<bool>,
    action_status: Option<&str>,
    priority: Option<&str>,
) -> Result<Vec<UserMessageDto>, Error> {
    let mut query = String::from(
        r#"
        SELECT 
            m.uuid AS message_uuid,
            m.message_type,
            m.title,
            m.content,
            m.sender_uuid,
            m.related_type,
            m.related_uuid,
            m.metadata,
            m.priority,
            m.created_at AS message_created_at,
            um.is_read,
            um.read_at,
            um.action_status,
            um.action_at,
            ui.nickname AS sender_name,
            ui.email AS sender_email
        FROM user_messages um
        INNER JOIN messages m ON um.message_uuid = m.uuid
        LEFT JOIN user_infos ui ON m.sender_uuid = ui.user_uuid
        WHERE um.user_uuid = $1 AND m.deleted_at IS NULL
        "#,
    );

    let mut param_count = 1;

    if message_type.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND m.message_type = ${}", param_count));
    }

    if is_read.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND um.is_read = ${}", param_count));
    }

    if action_status.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND um.action_status = ${}", param_count));
    }

    if priority.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND m.priority = ${}", param_count));
    }

    query.push_str(" ORDER BY m.created_at DESC");
    param_count += 1;
    query.push_str(&format!(" LIMIT ${}", param_count));
    param_count += 1;
    query.push_str(&format!(" OFFSET ${}", param_count));

    // 构建查询参数
    let mut query_builder = sqlx::query_as::<_, UserMessageDto>(&query);
    query_builder = query_builder.bind(user_uuid);

    if let Some(mt) = message_type {
        query_builder = query_builder.bind(mt);
    }
    if let Some(ir) = is_read {
        query_builder = query_builder.bind(ir);
    }
    if let Some(as_) = action_status {
        query_builder = query_builder.bind(as_);
    }
    if let Some(p) = priority {
        query_builder = query_builder.bind(p);
    }

    query_builder = query_builder.bind(limit);
    query_builder = query_builder.bind(offset);

    let recs = query_builder.fetch_all(pool).await?;

    Ok(recs)
}

/// 查询用户消息总数
pub async fn fetch_user_messages_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    message_type: Option<&str>,
    is_read: Option<bool>,
    action_status: Option<&str>,
    priority: Option<&str>,
) -> Result<i64, Error> {
    let mut query = String::from(
        r#"
        SELECT COUNT(*)
        FROM user_messages um
        INNER JOIN messages m ON um.message_uuid = m.uuid
        WHERE um.user_uuid = $1 AND m.deleted_at IS NULL
        "#,
    );

    let mut param_count = 1;

    if message_type.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND m.message_type = ${}", param_count));
    }

    if is_read.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND um.is_read = ${}", param_count));
    }

    if action_status.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND um.action_status = ${}", param_count));
    }

    if priority.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND m.priority = ${}", param_count));
    }

    let mut query_builder = sqlx::query_scalar::<_, i64>(&query);
    query_builder = query_builder.bind(user_uuid);

    if let Some(mt) = message_type {
        query_builder = query_builder.bind(mt);
    }
    if let Some(ir) = is_read {
        query_builder = query_builder.bind(ir);
    }
    if let Some(as_) = action_status {
        query_builder = query_builder.bind(as_);
    }
    if let Some(p) = priority {
        query_builder = query_builder.bind(p);
    }

    let count = query_builder.fetch_one(pool).await?;

    Ok(count)
}

/// 标记消息为已读
pub async fn mark_message_read(
    pool: &Pool<Postgres>,
    message_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_messages
        SET is_read = TRUE, read_at = NOW(), updated_at = NOW()
        WHERE message_uuid = $1 AND user_uuid = $2 AND is_read = FALSE
        "#,
    )
    .bind(message_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量标记消息为已读
pub async fn batch_mark_messages_read(
    pool: &Pool<Postgres>,
    message_uuids: &[Uuid],
    user_uuid: Uuid,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    for message_uuid in message_uuids {
        sqlx::query(
            r#"
            UPDATE user_messages
            SET is_read = TRUE, read_at = NOW(), updated_at = NOW()
            WHERE message_uuid = $1 AND user_uuid = $2 AND is_read = FALSE
            "#,
        )
        .bind(message_uuid)
        .bind(user_uuid)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// 处理消息（接受/拒绝）
pub async fn handle_message(
    pool: &Pool<Postgres>,
    message_uuid: Uuid,
    user_uuid: Uuid,
    action: &str,
) -> Result<(), Error> {
    let action_status = match action {
        "accept" => "accepted",
        "reject" => "rejected",
        _ => return Err(Error::RowNotFound),
    };

    sqlx::query(
        r#"
        UPDATE user_messages
        SET action_status = $1, action_at = NOW(), updated_at = NOW()
        WHERE message_uuid = $2 AND user_uuid = $3
        "#,
    )
    .bind(action_status)
    .bind(message_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 获取用户消息统计
pub async fn fetch_user_message_stats(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<(i64, i64, std::collections::HashMap<String, i64>), Error> {
    // 总消息数
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM user_messages um
        INNER JOIN messages m ON um.message_uuid = m.uuid
        WHERE um.user_uuid = $1 AND m.deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    // 未读消息数
    let unread: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM user_messages um
        INNER JOIN messages m ON um.message_uuid = m.uuid
        WHERE um.user_uuid = $1 AND um.is_read = FALSE AND m.deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    // 按类型统计
    let type_stats: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT m.message_type, COUNT(*) as count
        FROM user_messages um
        INNER JOIN messages m ON um.message_uuid = m.uuid
        WHERE um.user_uuid = $1 AND m.deleted_at IS NULL
        GROUP BY m.message_type
        "#,
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    let mut by_type = std::collections::HashMap::new();
    for (msg_type, count) in type_stats {
        by_type.insert(msg_type, count);
    }

    Ok((total, unread, by_type))
}

/// 删除消息（软删除）
pub async fn delete_message(pool: &Pool<Postgres>, message_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE messages
        SET deleted_at = NOW(), status = 'deleted', updated_at = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(message_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 UUID 查询消息
pub async fn fetch_message_by_uuid(
    pool: &Pool<Postgres>,
    message_uuid: Uuid,
) -> Result<Option<MessageDto>, Error> {
    let rec = sqlx::query_as::<_, MessageDto>(
        r#"
        SELECT id, uuid, message_type, title, content, sender_uuid, recipient_type,
               related_type, related_uuid, metadata, status, priority,
               created_at, updated_at, deleted_at
        FROM messages
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(message_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}
