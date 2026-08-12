use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{
    EnvironmentAccountRowDto, EnvironmentConfigDto, EnvironmentCookieDto, EnvironmentDto,
    EnvironmentRowDto, EnvironmentTagRowDto, EnvironmentUrlDto, GroupDto, GroupRowDto, ProxyRowDto,
    TagDto, TemplateDto,
};
use crate::entitys::CookieInput;

// ============ Groups ============

/// 创建分组
pub async fn insert_group(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    name: &str,
    description: Option<&str>,
    created_by: Uuid,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO groups (workspace_uuid, team_uuid, name, description, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING uuid;
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(name)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询分组列表（工作空间级别）
pub async fn fetch_groups(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    offset: i64,
    limit: i64,
) -> Result<Vec<GroupDto>, Error> {
    let recs = sqlx::query_as::<_, GroupDto>(
        r#"
        SELECT g.id, g.uuid, g.workspace_uuid, g.team_uuid, t.name AS team_name,
               g.name, g.description, g.sort_order,
               g.created_by, ui.nickname AS created_by_name,
               (SELECT COUNT(*) FROM environments e WHERE e.group_uuid = g.uuid AND e.deleted_at IS NULL) AS environments_count,
               g.created_at, g.updated_at, g.deleted_at
        FROM groups g
        LEFT JOIN teams t ON g.team_uuid = t.uuid
        LEFT JOIN user_infos ui ON g.created_by = ui.user_uuid
        WHERE g.workspace_uuid = $1 AND g.team_uuid = $2 AND g.deleted_at IS NULL
        ORDER BY g.sort_order, g.name
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据 UUID 查询分组
pub async fn fetch_group_by_uuid(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
) -> Result<Option<GroupDto>, Error> {
    let rec = sqlx::query_as::<_, GroupDto>(
        r#"
        SELECT g.id, g.uuid, g.workspace_uuid, g.team_uuid, t.name AS team_name,
               g.name, g.description, g.sort_order,
               g.created_by, ui.nickname AS created_by_name,
               (SELECT COUNT(*) FROM environments e WHERE e.group_uuid = g.uuid AND e.deleted_at IS NULL) AS environments_count,
               g.created_at, g.updated_at, g.deleted_at
        FROM groups g
        LEFT JOIN teams t ON g.team_uuid = t.uuid
        LEFT JOIN user_infos ui ON g.created_by = ui.user_uuid
        WHERE g.uuid = $1 AND g.deleted_at IS NULL
        "#,
    )
    .bind(group_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新分组
pub async fn update_group(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    sort_order: Option<i32>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE groups
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            sort_order = COALESCE($3, sort_order)
        WHERE uuid = $4 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(sort_order)
    .bind(group_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除分组
pub async fn delete_group(pool: &Pool<Postgres>, group_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE groups SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(group_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Tags ============

/// 创建标签
pub async fn insert_tag(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    name: &str,
    color: Option<&str>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO tags (user_uuid, team_uuid, name, color)
        VALUES ($1, $2, $3, $4)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name)
    .bind(color.unwrap_or("gray"))
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询标签列表
pub async fn fetch_tags(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
) -> Result<Vec<TagDto>, Error> {
    let recs = sqlx::query_as::<_, TagDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, color, sort_order,
               environments_count, created_at, updated_at, deleted_at
        FROM tags
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2))
          AND deleted_at IS NULL
        ORDER BY sort_order, name
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据 UUID 查询标签
pub async fn fetch_tag_by_uuid(
    pool: &Pool<Postgres>,
    tag_uuid: Uuid,
) -> Result<Option<TagDto>, Error> {
    let rec = sqlx::query_as::<_, TagDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, color, sort_order,
               environments_count, created_at, updated_at, deleted_at
        FROM tags
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(tag_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新标签
pub async fn update_tag(
    pool: &Pool<Postgres>,
    tag_uuid: Uuid,
    name: Option<&str>,
    color: Option<&str>,
    sort_order: Option<i32>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE tags
        SET name = COALESCE($1, name),
            color = COALESCE($2, color),
            sort_order = COALESCE($3, sort_order)
        WHERE uuid = $4 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(color)
    .bind(sort_order)
    .bind(tag_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除标签
pub async fn delete_tag(pool: &Pool<Postgres>, tag_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE tags SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(tag_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Environments ============

/// 创建环境
pub async fn insert_environment(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    team_uuid: Uuid,
    name: &str,
    description: Option<&str>,
    group_uuid: Option<Uuid>,
    proxy_uuid: Option<Uuid>,
    system_info: Option<&str>,
    kernel_info: Option<&str>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO environments (workspace_uuid, user_uuid, team_uuid, name, description,
                                   group_uuid, proxy_uuid, system_info, kernel_info)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING uuid;
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name)
    .bind(description)
    .bind(group_uuid)
    .bind(proxy_uuid)
    .bind(system_info)
    .bind(kernel_info)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询环境列表（基础信息）
pub async fn fetch_environments_base(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
    status: Option<&str>,
    keyword: Option<&str>,
    tag_uuids: Option<&[Uuid]>,
    offset: i64,
    limit: i64,
) -> Result<Vec<EnvironmentRowDto>, Error> {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            e.id, e.uuid, e.workspace_uuid, e.user_uuid, e.team_uuid, e.name, e.description, e.status,
            e.system_info, e.kernel_info, e.fingerprint_summary,
            e.group_uuid, e.proxy_uuid,
            e.last_opened_at, e.created_at, e.updated_at
        FROM environments e
        "#,
    );

    // 如果有标签过滤，需要 JOIN environment_tags 表
    if tag_uuids.is_some() {
        query.push_str(" LEFT JOIN environment_tags et ON e.uuid = et.environment_uuid ");
    }

    query.push_str(
        r#"
        WHERE e.workspace_uuid = $1
          AND e.team_uuid = $2
          AND e.deleted_at IS NULL
        "#,
    );

    let mut param_index = 3;
    let mut conditions = Vec::new();

    // 分组过滤
    if group_uuid.is_some() {
        conditions.push(format!("e.group_uuid = ${}", param_index));
        param_index += 1;
    }

    // 状态过滤
    if status.is_some() {
        conditions.push(format!("e.status = ${}", param_index));
        param_index += 1;
    }

    // 关键词搜索
    if keyword.is_some() {
        conditions.push(format!(
            "(e.name ILIKE ${} OR e.uuid::text ILIKE ${})",
            param_index, param_index
        ));
        param_index += 1;
    }

    // 标签过滤
    if let Some(tags) = tag_uuids {
        if !tags.is_empty() {
            conditions.push(format!("et.tag_uuid = ANY(${})", param_index));
            param_index += 1;
        }
    }

    // 添加所有条件
    for condition in conditions {
        query.push_str(&format!(" AND {}", condition));
    }

    query.push_str(&format!(
        " ORDER BY e.created_at DESC LIMIT ${} OFFSET ${}",
        param_index,
        param_index + 1
    ));

    // 构建查询
    let mut sql_query = sqlx::query_as::<_, EnvironmentRowDto>(&query)
        .bind(workspace_uuid)
        .bind(team_uuid);

    // 绑定参数
    if let Some(g) = group_uuid {
        sql_query = sql_query.bind(g);
    }
    if let Some(s) = status {
        sql_query = sql_query.bind(s);
    }
    if let Some(k) = keyword {
        let search_pattern = format!("%{}%", k);
        sql_query = sql_query.bind(search_pattern);
    }
    if let Some(tags) = tag_uuids {
        if !tags.is_empty() {
            sql_query = sql_query.bind(tags);
        }
    }

    sql_query = sql_query.bind(limit).bind(offset);

    let recs = sql_query.fetch_all(pool).await?;

    Ok(recs)
}

/// 查询环境列表（基础）
pub async fn fetch_environments(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<EnvironmentDto>, Error> {
    let recs = sqlx::query_as::<_, EnvironmentDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, team_uuid, name, description,
               status, group_uuid, proxy_uuid, system_info, kernel_info, fingerprint_summary,
               last_opened_at, created_at, updated_at, deleted_at
        FROM environments
        WHERE workspace_uuid = $1 AND team_uuid = $2
          AND ($3::uuid IS NULL OR group_uuid = $3)
          AND ($4::varchar IS NULL OR status = $4)
          AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(group_uuid)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询环境总数
pub async fn fetch_environments_count(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
    status: Option<&str>,
    keyword: Option<&str>,
    tag_uuids: Option<&[Uuid]>,
) -> Result<i64, Error> {
    let mut query = String::from(
        r#"
        SELECT COUNT(DISTINCT e.id) FROM environments e
        "#,
    );

    // 如果有标签过滤，需要 JOIN environment_tags 表
    if tag_uuids.is_some() {
        query.push_str(" LEFT JOIN environment_tags et ON e.uuid = et.environment_uuid ");
    }

    query.push_str(
        r#"
        WHERE e.workspace_uuid = $1
          AND e.team_uuid = $2
          AND e.deleted_at IS NULL
        "#,
    );

    let mut param_index = 3;
    let mut conditions = Vec::new();

    // 分组过滤
    if group_uuid.is_some() {
        conditions.push(format!("e.group_uuid = ${}", param_index));
        param_index += 1;
    }

    // 状态过滤
    if status.is_some() {
        conditions.push(format!("e.status = ${}", param_index));
        param_index += 1;
    }

    // 关键词搜索
    if keyword.is_some() {
        conditions.push(format!(
            "(e.name ILIKE ${} OR e.uuid::text ILIKE ${})",
            param_index, param_index
        ));
        param_index += 1;
    }

    // 标签过滤
    if let Some(tags) = tag_uuids {
        if !tags.is_empty() {
            conditions.push(format!("et.tag_uuid = ANY(${})", param_index));
        }
    }

    // 添加所有条件
    for condition in conditions {
        query.push_str(&format!(" AND {}", condition));
    }

    // 构建查询
    let mut sql_query = sqlx::query_scalar::<_, i64>(&query).bind(workspace_uuid).bind(team_uuid);

    // 绑定参数
    if let Some(g) = group_uuid {
        sql_query = sql_query.bind(g);
    }
    if let Some(s) = status {
        sql_query = sql_query.bind(s);
    }
    if let Some(k) = keyword {
        let search_pattern = format!("%{}%", k);
        sql_query = sql_query.bind(search_pattern);
    }
    if let Some(tags) = tag_uuids {
        if !tags.is_empty() {
            sql_query = sql_query.bind(tags);
        }
    }

    let count = sql_query.fetch_one(pool).await?;

    Ok(count)
}

/// 根据 UUID 查询环境（不带工作空间过滤，用于内部查询）
pub async fn fetch_environment_by_uuid_unfiltered(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Option<EnvironmentDto>, Error> {
    let rec = sqlx::query_as::<_, EnvironmentDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, team_uuid, name, description,
               status, group_uuid, proxy_uuid, system_info, kernel_info, fingerprint_summary,
               last_opened_at, created_at, updated_at, deleted_at
        FROM environments
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(env_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 根据 UUID 查询环境（带工作空间过滤）
pub async fn fetch_environment_by_uuid(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    env_uuid: Uuid,
) -> Result<Option<EnvironmentDto>, Error> {
    let rec = sqlx::query_as::<_, EnvironmentDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, team_uuid, name, description,
               status, group_uuid, proxy_uuid, system_info, kernel_info, fingerprint_summary,
               last_opened_at, created_at, updated_at, deleted_at
        FROM environments
        WHERE uuid = $1 AND workspace_uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(env_uuid)
    .bind(workspace_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新环境基础信息
pub async fn update_environment(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    group_uuid: Option<Uuid>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            group_uuid = $3
        WHERE uuid = $4 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(group_uuid)
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新环境状态
pub async fn update_environment_status(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    status: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments SET status = $1
        WHERE uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新环境代理
pub async fn update_environment_proxy(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    proxy_uuid: Option<Uuid>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments SET proxy_uuid = $1
        WHERE uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuid)
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新环境最后打开时间
pub async fn update_environment_last_opened(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments SET last_opened_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除环境
pub async fn delete_environment(pool: &Pool<Postgres>, env_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量软删除环境
pub async fn batch_delete_environments(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        UPDATE environments SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(env_uuids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Recycle Bin ============

/// 查询回收站环境列表（已删除但未永久删除）
pub async fn fetch_deleted_environments_base(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
    keyword: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<EnvironmentRowDto>, Error> {
    let mut query = String::from(
        r#"
        SELECT DISTINCT
            e.id, e.uuid, e.workspace_uuid, e.user_uuid, e.team_uuid, e.name, e.description, e.status,
            e.system_info, e.kernel_info, e.fingerprint_summary,
            e.group_uuid, e.proxy_uuid,
            e.last_opened_at, e.created_at, e.updated_at, e.deleted_at
        FROM environments e
        WHERE e.workspace_uuid = $1
          AND e.team_uuid = $2
          AND e.deleted_at IS NOT NULL
        "#,
    );

    let mut param_index = 3;
    let mut conditions = Vec::new();

    // 分组过滤
    if group_uuid.is_some() {
        conditions.push(format!("e.group_uuid = ${}", param_index));
        param_index += 1;
    }

    // 关键词搜索
    if keyword.is_some() {
        conditions.push(format!(
            "(e.name ILIKE ${} OR e.uuid::text ILIKE ${})",
            param_index, param_index
        ));
        param_index += 1;
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY e.deleted_at DESC LIMIT $");
    query.push_str(&param_index.to_string());
    param_index += 1;
    query.push_str(" OFFSET $");
    query.push_str(&param_index.to_string());

    let mut q = sqlx::query_as::<_, EnvironmentRowDto>(&query)
        .bind(workspace_uuid)
        .bind(team_uuid);

    if let Some(gid) = group_uuid {
        q = q.bind(gid);
    }

    if let Some(kw) = keyword {
        let pattern = format!("%{}%", kw);
        q = q.bind(pattern);
    }

    q = q.bind(limit).bind(offset);

    let recs = q.fetch_all(pool).await?;
    Ok(recs)
}

/// 统计回收站环境总数
pub async fn fetch_deleted_environments_count(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
    keyword: Option<&str>,
) -> Result<i64, Error> {
    let mut query = String::from(
        r#"
        SELECT COUNT(DISTINCT e.id)
        FROM environments e
        WHERE e.workspace_uuid = $1
          AND e.team_uuid = $2
          AND e.deleted_at IS NOT NULL
        "#,
    );

    let mut param_index = 3;
    let mut conditions = Vec::new();

    if group_uuid.is_some() {
        conditions.push(format!("e.group_uuid = ${}", param_index));
        param_index += 1;
    }

    if keyword.is_some() {
        conditions.push(format!(
            "(e.name ILIKE ${} OR e.uuid::text ILIKE ${})",
            param_index, param_index
        ));
        param_index += 1;
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    let mut q = sqlx::query_scalar::<_, i64>(&query).bind(workspace_uuid).bind(team_uuid);

    if let Some(gid) = group_uuid {
        q = q.bind(gid);
    }

    if let Some(kw) = keyword {
        let pattern = format!("%{}%", kw);
        q = q.bind(pattern);
    }

    let count = q.fetch_one(pool).await?;
    Ok(count)
}

/// 恢复环境（将 deleted_at 设为 NULL）
pub async fn restore_environment(pool: &Pool<Postgres>, env_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE environments SET deleted_at = NULL
        WHERE uuid = $1 AND deleted_at IS NOT NULL
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量恢复环境
pub async fn batch_restore_environments(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        UPDATE environments SET deleted_at = NULL
        WHERE uuid = ANY($1) AND deleted_at IS NOT NULL
        "#,
    )
    .bind(env_uuids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// 永久删除环境（真正的 DELETE）
pub async fn permanent_delete_environment(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<(), Error> {
    // 先删除关联数据
    sqlx::query("DELETE FROM environment_tags WHERE environment_uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_urls WHERE environment_uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_cookies WHERE environment_uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_configs WHERE environment_uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_accounts WHERE environment_uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    // `environment_extensions` 已在扩展系统重构后移除。
    // 环境的扩展现在通过 user/team/group 绑定动态合并，不再需要清理环境级关联。

    // 最后删除环境本身
    sqlx::query("DELETE FROM environments WHERE uuid = $1")
        .bind(env_uuid)
        .execute(pool)
        .await?;

    Ok(())
}

/// 批量永久删除环境
pub async fn batch_permanent_delete_environments(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<u64, Error> {
    // 先删除关联数据
    sqlx::query("DELETE FROM environment_tags WHERE environment_uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_urls WHERE environment_uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_cookies WHERE environment_uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_configs WHERE environment_uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM environment_accounts WHERE environment_uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    // `environment_extensions` 已在扩展系统重构后移除。
    // 环境的扩展现在通过 user/team/group 绑定动态合并，不再需要清理环境级关联。

    // 最后删除环境本身
    let result = sqlx::query("DELETE FROM environments WHERE uuid = ANY($1)")
        .bind(env_uuids)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

// ============ Environment Configs ============

/// 创建或更新环境配置
pub async fn upsert_environment_config(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    window_info: &serde_json::Value,
    basic_settings: &serde_json::Value,
    fingerprint_settings: &serde_json::Value,
    device_settings: &serde_json::Value,
    preference_settings: &serde_json::Value,
    project_metadata: &serde_json::Value,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO environment_configs (environment_uuid, window_info, basic_settings,
                                          fingerprint_settings, device_settings, preference_settings,
                                          project_metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (environment_uuid) DO UPDATE SET
            window_info = $2,
            basic_settings = $3,
            fingerprint_settings = $4,
            device_settings = $5,
            preference_settings = $6,
            project_metadata = $7
        RETURNING id;
        "#,
    )
    .bind(env_uuid)
    .bind(window_info)
    .bind(basic_settings)
    .bind(fingerprint_settings)
    .bind(device_settings)
    .bind(preference_settings)
    .bind(project_metadata)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 查询环境配置
pub async fn fetch_environment_config(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Option<EnvironmentConfigDto>, Error> {
    let rec = sqlx::query_as::<_, EnvironmentConfigDto>(
        r#"
        SELECT id, environment_uuid, window_info, basic_settings, fingerprint_settings,
               device_settings, preference_settings, project_metadata, created_at, updated_at
        FROM environment_configs
        WHERE environment_uuid = $1
        "#,
    )
    .bind(env_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 批量查询环境配置
pub async fn fetch_environment_configs_by_uuids(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<Vec<EnvironmentConfigDto>, Error> {
    if env_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, EnvironmentConfigDto>(
        r#"
        SELECT id, environment_uuid, window_info, basic_settings, fingerprint_settings,
               device_settings, preference_settings, project_metadata, created_at, updated_at
        FROM environment_configs
        WHERE environment_uuid = ANY($1)
        "#,
    )
    .bind(env_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

// ============ Environment Tags ============

/// 为环境添加标签
pub async fn insert_environment_tag(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    tag_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO environment_tags (environment_uuid, tag_uuid)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING;
        "#,
    )
    .bind(env_uuid)
    .bind(tag_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 移除环境的标签
pub async fn remove_environment_tag(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    tag_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_tags
        WHERE environment_uuid = $1 AND tag_uuid = $2;
        "#,
    )
    .bind(env_uuid)
    .bind(tag_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 清除环境的所有标签
pub async fn clear_environment_tags(pool: &Pool<Postgres>, env_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_tags
        WHERE environment_uuid = $1;
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询环境的所有标签
pub async fn fetch_environment_tags(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Vec<TagDto>, Error> {
    let recs = sqlx::query_as::<_, TagDto>(
        r#"
        SELECT t.id, t.uuid, t.user_uuid, t.team_uuid, t.name, t.color, t.sort_order,
               t.environments_count, t.created_at, t.updated_at, t.deleted_at
        FROM tags t
        INNER JOIN environment_tags et ON t.uuid = et.tag_uuid
        WHERE et.environment_uuid = $1 AND t.deleted_at IS NULL
        ORDER BY t.sort_order, t.name
        "#,
    )
    .bind(env_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询环境标签（完整标签信息）
pub async fn fetch_tags_for_environments(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<Vec<EnvironmentTagRowDto>, Error> {
    if env_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, EnvironmentTagRowDto>(
        r#"
        SELECT 
            et.environment_uuid,
            t.id as tag_id,
            t.uuid as tag_uuid, 
            t.name as tag_name, 
            t.color as tag_color,
            t.sort_order as tag_sort_order,
            t.user_uuid as tag_user_uuid,
            t.team_uuid as tag_team_uuid,
            t.environments_count as tag_environments_count,
            t.created_at as tag_created_at,
            t.updated_at as tag_updated_at,
            t.deleted_at as tag_deleted_at
        FROM environment_tags et
        INNER JOIN tags t ON et.tag_uuid = t.uuid
        WHERE et.environment_uuid = ANY($1) AND t.deleted_at IS NULL
        ORDER BY t.sort_order, t.name
        "#,
    )
    .bind(env_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询环境账号（完整账号信息，排除敏感数据）
pub async fn fetch_accounts_for_environments(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<Vec<EnvironmentAccountRowDto>, Error> {
    if env_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, EnvironmentAccountRowDto>(
        r#"
        SELECT 
            ea.environment_uuid,
            pa.id as account_id,
            pa.uuid as account_uuid, 
            pa.platform_url,
            pa.platform_name, 
            pa.account,
            pa.status as account_status,
            pa.remark
        FROM environment_accounts ea
        INNER JOIN platform_accounts pa ON ea.account_uuid = pa.uuid
        WHERE ea.environment_uuid = ANY($1) AND pa.deleted_at IS NULL
        ORDER BY ea.sort_order
        "#,
    )
    .bind(env_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询分组
pub async fn fetch_groups_by_uuids(
    pool: &Pool<Postgres>,
    group_uuids: &[Uuid],
) -> Result<Vec<GroupRowDto>, Error> {
    if group_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, GroupRowDto>(
        r#"
        SELECT id, uuid, name, description, sort_order
        FROM groups
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(group_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询代理
pub async fn fetch_proxies_by_uuids(
    pool: &Pool<Postgres>,
    proxy_uuids: &[Uuid],
) -> Result<Vec<ProxyRowDto>, Error> {
    if proxy_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, ProxyRowDto>(
        r#"
        SELECT id, uuid, name, host, port, proxy_type,
               username, password,
               country, city, status, latency, last_check_ip
        FROM proxies
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

// ============ Templates ============

/// 创建模板
pub async fn insert_template(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    name: &str,
    description: Option<&str>,
    is_public: bool,
    system_info: Option<&str>,
    kernel_info: Option<&str>,
    config_json: &serde_json::Value,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO templates (user_uuid, team_uuid, name, description, is_public,
                                system_info, kernel_info, config_json)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name)
    .bind(description)
    .bind(is_public)
    .bind(system_info)
    .bind(kernel_info)
    .bind(config_json)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询模板列表
pub async fn fetch_templates(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    is_public: Option<bool>,
    offset: i64,
    limit: i64,
) -> Result<Vec<TemplateDto>, Error> {
    let recs = sqlx::query_as::<_, TemplateDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, description, is_public,
               system_info, kernel_info, config_json, usage_count, created_at, updated_at, deleted_at
        FROM templates
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2) OR is_public = TRUE)
          AND ($3::boolean IS NULL OR is_public = $3)
          AND deleted_at IS NULL
        ORDER BY usage_count DESC, created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(is_public)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询模板总数
pub async fn fetch_templates_count(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    is_public: Option<bool>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM templates
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2) OR is_public = TRUE)
          AND ($3::boolean IS NULL OR is_public = $3)
          AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(is_public)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询模板
pub async fn fetch_template_by_uuid(
    pool: &Pool<Postgres>,
    template_uuid: Uuid,
) -> Result<Option<TemplateDto>, Error> {
    let rec = sqlx::query_as::<_, TemplateDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, description, is_public,
               system_info, kernel_info, config_json, usage_count, created_at, updated_at, deleted_at
        FROM templates
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(template_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新模板
pub async fn update_template(
    pool: &Pool<Postgres>,
    template_uuid: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    is_public: Option<bool>,
    config_json: Option<&serde_json::Value>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE templates
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            is_public = COALESCE($3, is_public),
            config_json = COALESCE($4, config_json)
        WHERE uuid = $5 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(is_public)
    .bind(config_json)
    .bind(template_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 增加模板使用次数
pub async fn increment_template_usage(
    pool: &Pool<Postgres>,
    template_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE templates SET usage_count = usage_count + 1
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(template_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除模板
pub async fn delete_template(pool: &Pool<Postgres>, template_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE templates SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(template_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Environment URLs ============

/// 添加环境 URL
pub async fn insert_environment_url(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    url: &str,
    title: Option<&str>,
    sort_order: Option<i32>,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO environment_urls (environment_uuid, url, title, sort_order)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        "#,
    )
    .bind(env_uuid)
    .bind(url)
    .bind(title)
    .bind(sort_order.unwrap_or(0))
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 批量添加环境 URL
pub async fn batch_insert_environment_urls(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    urls: &[(String, Option<String>)],
) -> Result<i32, Error> {
    let mut count = 0;
    for (idx, (url, title)) in urls.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO environment_urls (environment_uuid, url, title, sort_order)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING;
            "#,
        )
        .bind(env_uuid)
        .bind(url)
        .bind(title.as_deref())
        .bind(idx as i32)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// 查询环境的所有 URL
pub async fn fetch_environment_urls(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Vec<EnvironmentUrlDto>, Error> {
    let recs = sqlx::query_as::<_, EnvironmentUrlDto>(
        r#"
        SELECT id, environment_uuid, url, title, sort_order, created_at
        FROM environment_urls
        WHERE environment_uuid = $1
        ORDER BY sort_order, id
        "#,
    )
    .bind(env_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询环境的所有 URL
pub async fn fetch_environment_urls_by_uuids(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<Vec<EnvironmentUrlDto>, Error> {
    if env_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, EnvironmentUrlDto>(
        r#"
        SELECT id, environment_uuid, url, title, sort_order, created_at
        FROM environment_urls
        WHERE environment_uuid = ANY($1)
        ORDER BY environment_uuid, sort_order, id
        "#,
    )
    .bind(env_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 删除环境的 URL
pub async fn delete_environment_url(pool: &Pool<Postgres>, url_id: i32) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_urls WHERE id = $1
        "#,
    )
    .bind(url_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 清空环境的所有 URL
pub async fn clear_environment_urls(pool: &Pool<Postgres>, env_uuid: Uuid) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM environment_urls WHERE environment_uuid = $1
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Environment Cookies ============

/// 添加环境 Cookie
pub async fn insert_environment_cookie(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    site_input: &str,
    domain: &str,
    name: &str,
    value: &str,
    path: Option<&str>,
    http_only: Option<bool>,
    secure: Option<bool>,
    same_site: Option<&str>,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO environment_cookies (environment_uuid, site_input, domain, name, value, path, http_only, secure, same_site)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id;
        "#,
    )
    .bind(env_uuid)
    .bind(site_input)
    .bind(domain)
    .bind(name)
    .bind(value)
    .bind(path.unwrap_or("/"))
    .bind(http_only.unwrap_or(false))
    .bind(secure.unwrap_or(false))
    .bind(same_site.unwrap_or("Lax"))
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 批量添加环境 Cookies
pub async fn batch_insert_environment_cookies(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    cookies: &[CookieInput],
) -> Result<i32, Error> {
    let mut count = 0;
    for cookie in cookies {
        sqlx::query(
            r#"
            INSERT INTO environment_cookies (environment_uuid, site_input, domain, name, value, path, http_only, secure, same_site)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT DO NOTHING;
            "#,
        )
        .bind(env_uuid)
        .bind(&cookie.site_input)
        .bind(&cookie.domain)
        .bind(&cookie.name)
        .bind(&cookie.value)
        .bind(cookie.path.as_deref().unwrap_or("/"))
        .bind(cookie.http_only.unwrap_or(false))
        .bind(cookie.secure.unwrap_or(false))
        .bind(cookie.same_site.as_deref().unwrap_or("Lax"))
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// 查询环境的所有 Cookies
pub async fn fetch_environment_cookies(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Vec<EnvironmentCookieDto>, Error> {
    let recs = sqlx::query_as::<_, EnvironmentCookieDto>(
        r#"
        SELECT id, environment_uuid, site_input, domain, name, value, path, expires_at, http_only, secure, same_site, created_at
        FROM environment_cookies
        WHERE environment_uuid = $1
        ORDER BY site_input, domain, name
        "#,
    )
    .bind(env_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 批量查询环境的所有 Cookies
pub async fn fetch_environment_cookies_by_uuids(
    pool: &Pool<Postgres>,
    env_uuids: &[Uuid],
) -> Result<Vec<EnvironmentCookieDto>, Error> {
    if env_uuids.is_empty() {
        return Ok(vec![]);
    }

    let recs = sqlx::query_as::<_, EnvironmentCookieDto>(
        r#"
        SELECT id, environment_uuid, site_input, domain, name, value, path, expires_at, http_only, secure, same_site, created_at
        FROM environment_cookies
        WHERE environment_uuid = ANY($1)
        ORDER BY environment_uuid, site_input, domain, name
        "#,
    )
    .bind(env_uuids)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 删除环境的 Cookie
pub async fn delete_environment_cookie(pool: &Pool<Postgres>, cookie_id: i32) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_cookies WHERE id = $1
        "#,
    )
    .bind(cookie_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 清空环境的所有 Cookies
pub async fn clear_environment_cookies(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM environment_cookies WHERE environment_uuid = $1
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
