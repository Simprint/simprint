use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::entitys::extensions::{CreateExtensionParams, UpdateExtensionParams};
use crate::dto::{
    ExtensionDto, GroupExtensionDto, TeamExtensionDto, UserExtensionDto,
};

// ============ Extensions CRUD ============

/// 创建扩展
pub async fn create_extension(
    pool: &Pool<Postgres>,
    params: CreateExtensionParams,
) -> Result<ExtensionDto, Error> {
    let rec = sqlx::query_as::<_, ExtensionDto>(
        r#"
        INSERT INTO extensions (
            extension_id, name, description, version, category, browser,
            developer, homepage, icon_url, download_url, file_size, downloads_count,
            permissions, rating, changelog, published_at, hash, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, 'active')
        RETURNING id, uuid, extension_id, name, description, version, category, browser,
                  developer, homepage, icon_url, download_url, file_size, downloads_count,
                  rating, permissions, status, changelog, published_at, hash, created_at, updated_at
        "#,
    )
    .bind(&params.extension_id)
    .bind(&params.name)
    .bind(&params.description)
    .bind(&params.version)
    .bind(&params.category)
    .bind(&params.browser)
    .bind(&params.developer)
    .bind(&params.homepage)
    .bind(&params.icon_url)
    .bind(&params.download_url)
    .bind(params.file_size)
    .bind(params.downloads_count)
    .bind(&params.permissions)
    .bind(params.rating)
    .bind(&params.changelog)
    .bind(params.published_at)
    .bind(&params.hash)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

/// 更新扩展
pub async fn update_extension(
    pool: &Pool<Postgres>,
    extension_id: &str,
    params: UpdateExtensionParams,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE extensions SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            version = COALESCE($4, version),
            category = COALESCE($5, category),
            developer = COALESCE($6, developer),
            homepage = COALESCE($7, homepage),
            icon_url = COALESCE($8, icon_url),
            download_url = COALESCE($9, download_url),
            file_size = COALESCE($10, file_size),
            downloads_count = COALESCE($11, downloads_count),
            permissions = COALESCE($12, permissions),
            rating = COALESCE($13, rating),
            changelog = COALESCE($14, changelog),
            published_at = COALESCE($15, published_at),
            hash = COALESCE($16, hash),
            updated_at = CURRENT_TIMESTAMP
        WHERE extension_id = $1
        "#,
    )
    .bind(extension_id)
    .bind(params.name)
    .bind(params.description)
    .bind(params.version)
    .bind(params.category)
    .bind(params.developer)
    .bind(params.homepage)
    .bind(params.icon_url)
    .bind(params.download_url)
    .bind(params.file_size)
    .bind(params.downloads_count)
    .bind(params.permissions)
    .bind(params.rating)
    .bind(params.changelog)
    .bind(params.published_at)
    .bind(params.hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据 extension_id 获取扩展（别名函数，用于同步检查）
pub async fn get_extension_by_extension_id(
    pool: &Pool<Postgres>,
    extension_id: &str,
) -> Result<Option<ExtensionDto>, Error> {
    fetch_extension_by_id(pool, extension_id).await
}

// ============ Extensions Query ============

/// 查询扩展列表
pub async fn fetch_extensions(
    pool: &Pool<Postgres>,
    keyword: Option<&str>,
    category: Option<&str>,
    sort_by: Option<&str>,
    sort_order: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ExtensionDto>, Error> {
    let order_clause = match (
        sort_by.unwrap_or("downloads"),
        sort_order.unwrap_or("desc").to_ascii_lowercase().as_str(),
    ) {
        ("rating", "asc") => "rating ASC NULLS LAST, created_at DESC",
        ("rating", _) => "rating DESC NULLS LAST, created_at DESC",
        ("name", "asc") => "name ASC, created_at DESC",
        ("name", _) => "name DESC, created_at DESC",
        ("newest", "asc") => "updated_at ASC NULLS LAST, created_at ASC",
        ("newest", _) => "updated_at DESC NULLS LAST, created_at DESC",
        ("downloads", "asc") => "downloads_count ASC NULLS LAST, created_at DESC",
        _ => "downloads_count DESC NULLS LAST, created_at DESC",
    };

    let query = format!(
        r#"
        SELECT id, uuid, extension_id, name, description, version, category, browser,
               developer, homepage, icon_url, download_url, file_size, downloads_count,
               rating, permissions, status, changelog, published_at, hash, created_at, updated_at
        FROM extensions
        WHERE status = 'active'
          AND ($1::varchar IS NULL OR name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%')
          AND ($2::varchar IS NULL OR category = $2)
        ORDER BY {}
        LIMIT $3 OFFSET $4
        "#,
        order_clause
    );

    let recs = sqlx::query_as::<_, ExtensionDto>(&query)
    .bind(keyword)
    .bind(category)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询扩展总数
pub async fn fetch_extensions_count(
    pool: &Pool<Postgres>,
    keyword: Option<&str>,
    category: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM extensions
        WHERE status = 'active'
          AND ($1::varchar IS NULL OR name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%')
          AND ($2::varchar IS NULL OR category = $2)
        "#,
    )
    .bind(keyword)
    .bind(category)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 extension_id 查询扩展
pub async fn fetch_extension_by_id(
    pool: &Pool<Postgres>,
    extension_id: &str,
) -> Result<Option<ExtensionDto>, Error> {
    let rec = sqlx::query_as::<_, ExtensionDto>(
        r#"
        SELECT id, uuid, extension_id, name, description, version, category, browser,
               developer, homepage, icon_url, download_url, file_size, downloads_count,
               rating, permissions, status, changelog, published_at, hash, created_at, updated_at
        FROM extensions
        WHERE extension_id = $1
        "#,
    )
    .bind(extension_id)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 获取扩展分类列表
pub async fn fetch_extension_categories(pool: &Pool<Postgres>) -> Result<Vec<String>, Error> {
    let categories: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT category FROM extensions
        WHERE status = 'active'
        ORDER BY category
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(categories.into_iter().map(|(c,)| c).collect())
}

// ============ User Extensions ============

/// 查询用户已安装的扩展
pub async fn fetch_user_extensions(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Vec<UserExtensionDto>, Error> {
    let recs = sqlx::query_as::<_, UserExtensionDto>(
        r#"
        SELECT id, user_uuid, extension_id, installed_version, status, installed_at, updated_at
        FROM user_extensions
        WHERE user_uuid = $1 AND status = 'active'
        ORDER BY installed_at DESC
        "#,
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 安装用户扩展
pub async fn insert_user_extension(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    extension_id: &str,
    version: &str,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO user_extensions (user_uuid, extension_id, installed_version, status)
        VALUES ($1, $2, $3, 'active')
        ON CONFLICT (user_uuid, extension_id) DO UPDATE SET
            installed_version = $3,
            status = 'active',
            updated_at = CURRENT_TIMESTAMP
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(extension_id)
    .bind(version)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 卸载用户扩展
pub async fn delete_user_extension(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    extension_id: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_extensions SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
        WHERE user_uuid = $1 AND extension_id = $2
        "#,
    )
    .bind(user_uuid)
    .bind(extension_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Team Extensions ============

/// 查询团队已安装的扩展
pub async fn fetch_team_extensions(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
) -> Result<Vec<TeamExtensionDto>, Error> {
    let recs = sqlx::query_as::<_, TeamExtensionDto>(
        r#"
        SELECT id, team_uuid, extension_id, installed_version, installed_by, status, installed_at, updated_at
        FROM team_extensions
        WHERE team_uuid = $1 AND status = 'active'
        ORDER BY installed_at DESC
        "#,
    )
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 安装团队扩展
pub async fn insert_team_extension(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    extension_id: &str,
    version: &str,
    installed_by: Uuid,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO team_extensions (team_uuid, extension_id, installed_version, installed_by, status)
        VALUES ($1, $2, $3, $4, 'active')
        ON CONFLICT (team_uuid, extension_id) DO UPDATE SET
            installed_version = $3,
            installed_by = $4,
            status = 'active',
            updated_at = CURRENT_TIMESTAMP
        RETURNING id;
        "#,
    )
    .bind(team_uuid)
    .bind(extension_id)
    .bind(version)
    .bind(installed_by)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 卸载团队扩展
pub async fn delete_team_extension(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    extension_id: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_extensions SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
        WHERE team_uuid = $1 AND extension_id = $2
        "#,
    )
    .bind(team_uuid)
    .bind(extension_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Group Extensions ============

/// 查询分组已安装的扩展
pub async fn fetch_group_extensions(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
) -> Result<Vec<GroupExtensionDto>, Error> {
    let recs = sqlx::query_as::<_, GroupExtensionDto>(
        r#"
        SELECT id, group_uuid, extension_id, installed_version, installed_by, status, is_team_shared, installed_at, updated_at
        FROM group_extensions
        WHERE group_uuid = $1 AND status = 'active'
        ORDER BY installed_at DESC
        "#,
    )
    .bind(group_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 安装分组扩展
pub async fn insert_group_extension(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
    extension_id: &str,
    version: &str,
    installed_by: Uuid,
    is_team_shared: bool,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO group_extensions (group_uuid, extension_id, installed_version, installed_by, is_team_shared, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        ON CONFLICT (group_uuid, extension_id) DO UPDATE SET
            installed_version = $3,
            installed_by = $4,
            is_team_shared = $5,
            status = 'active',
            updated_at = CURRENT_TIMESTAMP
        RETURNING id;
        "#,
    )
    .bind(group_uuid)
    .bind(extension_id)
    .bind(version)
    .bind(installed_by)
    .bind(is_team_shared)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 卸载分组扩展（指定分组）
pub async fn delete_group_extension(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
    extension_id: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE group_extensions SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
        WHERE group_uuid = $1 AND extension_id = $2
        "#,
    )
    .bind(group_uuid)
    .bind(extension_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 卸载分组扩展（根据扩展 ID 删除所有相关分组记录）
pub async fn delete_group_extensions_by_extension_id(
    pool: &Pool<Postgres>,
    extension_id: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE group_extensions SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
        WHERE extension_id = $1 AND status = 'active'
        "#,
    )
    .bind(extension_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 根据扩展 ID 查询所有关联的分组 UUID
pub async fn fetch_group_uuids_by_extension_id(
    pool: &Pool<Postgres>,
    extension_id: &str,
) -> Result<Vec<Uuid>, Error> {
    let recs = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT DISTINCT group_uuid
        FROM group_extensions
        WHERE extension_id = $1 AND status = 'active'
        "#,
    )
    .bind(extension_id)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据扩展 ID 查询团队共享的分组 UUID
pub async fn fetch_team_shared_group_uuids_by_extension_id(
    pool: &Pool<Postgres>,
    extension_id: &str,
) -> Result<Vec<Uuid>, Error> {
    let recs = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT DISTINCT group_uuid
        FROM group_extensions
        WHERE extension_id = $1 AND status = 'active' AND is_team_shared = true
        "#,
    )
    .bind(extension_id)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询用户相关的分组中安装的扩展
/// 包括用户个人分组（无 team_uuid，由用户创建）和用户所在团队的分组
pub async fn fetch_user_group_extensions(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<Vec<GroupExtensionDto>, Error> {
    let recs = sqlx::query_as::<_, GroupExtensionDto>(
        r#"
        SELECT ge.id, ge.group_uuid, ge.extension_id, ge.installed_version, ge.installed_by, ge.status, ge.is_team_shared, ge.installed_at, ge.updated_at
        FROM group_extensions ge
        INNER JOIN groups g ON ge.group_uuid = g.uuid
        WHERE ge.status = 'active'
          AND g.deleted_at IS NULL
          AND (g.team_uuid = $1 OR (g.team_uuid IS NULL AND g.created_by = $2))
        ORDER BY ge.installed_at DESC
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询团队相关的分组中安装的扩展
pub async fn fetch_team_group_extensions(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
) -> Result<Vec<GroupExtensionDto>, Error> {
    let recs = sqlx::query_as::<_, GroupExtensionDto>(
        r#"
        SELECT ge.id, ge.group_uuid, ge.extension_id, ge.installed_version, ge.installed_by, ge.status, ge.is_team_shared, ge.installed_at, ge.updated_at
        FROM group_extensions ge
        INNER JOIN groups g ON ge.group_uuid = g.uuid
        WHERE ge.status = 'active'
          AND ge.is_team_shared = true
          AND g.deleted_at IS NULL
          AND g.team_uuid = $1
        ORDER BY ge.installed_at DESC
        "#,
    )
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

// ============ User Team Extension Preferences ============

/// 查询用户禁用的团队插件列表
pub async fn fetch_user_disabled_team_extensions(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<Vec<String>, Error> {
    let recs = sqlx::query_scalar::<_, String>(
        r#"
        SELECT extension_id
        FROM user_team_extension_preferences
        WHERE user_uuid = $1 AND team_uuid = $2 AND is_disabled = true
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 设置用户对团队插件的禁用状态
pub async fn set_user_team_extension_preference(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Uuid,
    extension_id: &str,
    is_disabled: bool,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO user_team_extension_preferences (user_uuid, team_uuid, extension_id, is_disabled)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_uuid, team_uuid, extension_id) DO UPDATE SET
            is_disabled = $4,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(extension_id)
    .bind(is_disabled)
    .execute(pool)
    .await?;

    Ok(())
}

/// 删除用户对团队插件的偏好设置
pub async fn delete_user_team_extension_preference(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Uuid,
    extension_id: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM user_team_extension_preferences
        WHERE user_uuid = $1 AND team_uuid = $2 AND extension_id = $3
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(extension_id)
    .execute(pool)
    .await?;

    Ok(())
}
