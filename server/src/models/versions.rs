use crate::dto::versions::Version;
use crate::entitys::versions::{CreateVersionRequest, UpdateVersionRequest};
use chrono::{DateTime, Utc};
use sqlx::{Error, Pool, Postgres};

/// 插入新版本
pub async fn insert_version(
    pool: &Pool<Postgres>,
    request: &CreateVersionRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO versions (
            type_id, resource_name, version, name, notes, platform, url, hash,
            signature, install_path, file_size, pub_date,
            arch, package_format, requires_extract, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            $13, $14, $15, NOW()
        ) RETURNING id
    ";

    let result: (i32,) = sqlx::query_as(sql)
        .bind(request.type_id)
        .bind(&request.resource_name)
        .bind(&request.version)
        .bind(&request.name)
        .bind(&request.notes)
        .bind(&request.platform)
        .bind(&request.url)
        .bind(&request.hash)
        .bind(&request.signature)
        .bind(&request.install_path)
        .bind(request.file_size)
        .bind(request.pub_date)
        .bind(&request.arch)
        .bind(&request.package_format)
        .bind(request.requires_extract.unwrap_or(true))
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 根据ID查询版本
pub async fn query_version_by_id(pool: &Pool<Postgres>, id: i32) -> Result<Version, Error> {
    let version: Version = sqlx::query_as("SELECT * FROM versions WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(version)
}

/// 根据资源名称和版本号查询
pub async fn query_version_by_name_and_version(
    pool: &Pool<Postgres>,
    resource_name: &str,
    version: &str,
) -> Result<Version, Error> {
    let version_data: Version =
        sqlx::query_as("SELECT * FROM versions WHERE resource_name = $1 AND version = $2")
            .bind(resource_name)
            .bind(version)
            .fetch_one(pool)
            .await?;

    Ok(version_data)
}

/// 查询最新版本
pub async fn query_latest_version(
    pool: &Pool<Postgres>,
    resource_name: &str,
    platform: &str,
) -> Result<Option<Version>, Error> {
    let version: Option<Version> = sqlx::query_as(
        "SELECT * FROM versions 
         WHERE resource_name = $1 AND platform = $2 AND status = 'active'
         ORDER BY pub_date DESC LIMIT 1",
    )
    .bind(resource_name)
    .bind(platform)
    .fetch_optional(pool)
    .await?;

    Ok(version)
}

/// 查询版本列表（分页）
pub async fn query_versions(
    pool: &Pool<Postgres>,
    resource_name: Option<&str>,
    platform: Option<&str>,
    status: Option<&str>,
    page_num: i32,
    page_size: i32,
) -> Result<(i64, Vec<Version>), Error> {
    // 构建查询条件
    let mut where_clauses = vec!["deleted_at IS NULL".to_string()];
    let mut param_index = 1;

    if let Some(_name) = resource_name {
        where_clauses.push(format!("resource_name = ${}", param_index));
        param_index += 1;
    }
    if let Some(_plat) = platform {
        where_clauses.push(format!("platform = ${}", param_index));
        param_index += 1;
    }
    if let Some(_st) = status {
        where_clauses.push(format!("status = ${}", param_index));
        param_index += 1;
    }

    let where_sql = where_clauses.join(" AND ");

    // 获取总数
    let count_sql = format!("SELECT COUNT(*) FROM versions WHERE {}", where_sql);
    let count_result: (i64,) = match (resource_name, platform, status) {
        (Some(n), Some(p), Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(n)
                .bind(p)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (Some(n), Some(p), None) => {
            sqlx::query_as(&count_sql)
                .bind(n)
                .bind(p)
                .fetch_one(pool)
                .await?
        }
        (Some(n), None, Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(n)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (Some(n), None, None) => sqlx::query_as(&count_sql).bind(n).fetch_one(pool).await?,
        (None, Some(p), Some(s)) => {
            sqlx::query_as(&count_sql)
                .bind(p)
                .bind(s)
                .fetch_one(pool)
                .await?
        }
        (None, Some(p), None) => sqlx::query_as(&count_sql).bind(p).fetch_one(pool).await?,
        (None, None, Some(s)) => sqlx::query_as(&count_sql).bind(s).fetch_one(pool).await?,
        (None, None, None) => sqlx::query_as(&count_sql).fetch_one(pool).await?,
    };

    // 获取分页列表
    let list_sql = format!(
        "SELECT * FROM versions WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_sql,
        param_index,
        param_index + 1
    );
    let versions: Vec<Version> = match (resource_name, platform, status) {
        (Some(n), Some(p), Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(n)
                .bind(p)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(n), Some(p), None) => {
            sqlx::query_as(&list_sql)
                .bind(n)
                .bind(p)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(n), None, Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(n)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (Some(n), None, None) => {
            sqlx::query_as(&list_sql)
                .bind(n)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, Some(p), Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(p)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, Some(p), None) => {
            sqlx::query_as(&list_sql)
                .bind(p)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, None, Some(s)) => {
            sqlx::query_as(&list_sql)
                .bind(s)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
        (None, None, None) => {
            sqlx::query_as(&list_sql)
                .bind(page_size)
                .bind((page_num - 1) * page_size)
                .fetch_all(pool)
                .await?
        }
    };

    Ok((count_result.0, versions))
}

/// 更新版本
pub async fn update_version(
    pool: &Pool<Postgres>,
    id: i32,
    request: &UpdateVersionRequest,
) -> Result<bool, Error> {
    let sql = "
        UPDATE versions SET
            name = COALESCE($1, name),
            notes = COALESCE($2, notes),
            platform = COALESCE($3, platform),
            url = COALESCE($4, url),
            hash = COALESCE($5, hash),
            signature = COALESCE($6, signature),
            install_path = COALESCE($7, install_path),
            file_size = COALESCE($8, file_size),
            status = COALESCE($9, status),
            is_latest = COALESCE($10, is_latest),
            updated_at = NOW()
        WHERE id = $11
    ";

    let row = sqlx::query(sql)
        .bind(&request.name)
        .bind(&request.notes)
        .bind(&request.platform)
        .bind(&request.url)
        .bind(&request.hash)
        .bind(&request.signature)
        .bind(&request.install_path)
        .bind(request.file_size)
        .bind(&request.status)
        .bind(request.is_latest)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 软删除版本
pub async fn delete_version(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    // 非逻辑删除
    let sql = "DELETE FROM versions WHERE id = $1";
    let row = sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(row.rows_affected() == 1)
}

/// 设置某个资源为最新版本
pub async fn set_as_latest_version(
    pool: &Pool<Postgres>,
    type_id: i32,
    resource_name: &str,
    version_id: i32,
) -> Result<bool, Error> {
    let mut tx = pool.begin().await?;

    // 先取消其他版本的最新状态
    sqlx::query(
        "UPDATE versions SET is_latest = false, updated_at = NOW()
         WHERE type_id = $1 AND resource_name = $2 AND id != $3",
    )
    .bind(type_id)
    .bind(resource_name)
    .bind(version_id)
    .execute(&mut *tx)
    .await?;

    // 设置当前版本为最新
    let row = sqlx::query("UPDATE versions SET is_latest = true, updated_at = NOW() WHERE id = $1")
        .bind(version_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(row.rows_affected() == 1)
}

/// 查询所有激活版本类型对应平台的最新版本
/// 使用 is_latest 字段优先，按 (type_code, resource_name) 分组
/// 返回 (type_code, resource_name, Version) 元组列表
pub async fn query_all_active_latest_versions(
    pool: &Pool<Postgres>,
    platform: &str,
) -> Result<Vec<(String, String, Version)>, Error> {
    // 自定义结构体用于接收查询结果
    #[derive(sqlx::FromRow)]
    struct VersionWithTypeCode {
        type_code: String,
        // Version 的所有字段
        id: i32,
        type_id: i32,
        resource_name: String,
        version: String,
        name: Option<String>,
        notes: Option<String>,
        platform: Option<String>,
        url: Option<String>,
        hash: Option<String>,
        signature: Option<String>,
        install_path: Option<String>,
        file_size: Option<i32>,
        is_latest: bool,
        status: String,
        pub_date: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: Option<DateTime<Utc>>,
        deleted_at: Option<DateTime<Utc>>,
        arch: Option<String>,
        package_format: Option<String>,
        requires_extract: bool,
        entrypoint_template: Option<String>,
        extract_root: Option<String>,
    }

    let results: Vec<VersionWithTypeCode> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (vt.type_code, v.resource_name) 
            vt.type_code,
            v.id, v.type_id, v.resource_name, v.version, v.name, v.notes,
            v.platform, v.url, v.hash, v.signature, v.install_path, 
            v.file_size, v.is_latest, v.status, v.pub_date,
            v.created_at, v.updated_at, v.deleted_at,
            v.arch, v.package_format, v.requires_extract, v.entrypoint_template, v.extract_root
        FROM versions v
        INNER JOIN version_types vt ON v.type_id = vt.id
        WHERE vt.is_active = true
          AND v.platform = $1
          AND v.status = 'active'
          AND v.deleted_at IS NULL
        ORDER BY vt.type_code, v.resource_name, v.is_latest DESC, v.id DESC
        "#,
    )
    .bind(platform)
    .fetch_all(pool)
    .await?;

    // 转换为 (String, String, Version) 元组
    let converted: Vec<(String, String, Version)> = results
        .into_iter()
        .map(|r| {
            (
                r.type_code.clone(),
                r.resource_name.clone(),
                Version {
                    id: r.id,
                    type_id: r.type_id,
                    resource_name: r.resource_name,
                    version: r.version,
                    name: r.name.clone(),
                    notes: r.notes,
                    platform: r.platform,
                    url: r.url,
                    hash: r.hash,
                    signature: r.signature,
                    install_path: r.install_path,
                    file_size: r.file_size,
                    is_latest: r.is_latest,
                    status: r.status,
                    pub_date: r.pub_date,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    deleted_at: r.deleted_at,
                    arch: r.arch,
                    package_format: r.package_format,
                    requires_extract: r.requires_extract,
                    entrypoint_template: r.entrypoint_template,
                    extract_root: r.extract_root,
                },
            )
        })
        .collect();

    Ok(converted)
}

/// 查询浏览器内核类型的最新版本列表
/// type_code_filter 为 None 时筛选 type_code LIKE 'SIMPRINT_KERNEL_%'，为 Some 时精确匹配
/// 按 (type_code, resource_name) 分组，每组取最新版本
/// 最新判定：is_latest 优先，其次 COALESCE(pub_date, created_at)，最后 id
/// platform 为可选，None 表示不按平台过滤
pub async fn query_browser_kernel_latest_versions(
    pool: &Pool<Postgres>,
    platform: Option<&str>,
    type_code_filter: Option<&str>,
) -> Result<Vec<(String, String, Version)>, Error> {
    #[derive(sqlx::FromRow)]
    struct VersionWithTypeCode {
        type_code: String,
        id: i32,
        type_id: i32,
        resource_name: String,
        version: String,
        name: Option<String>,
        notes: Option<String>,
        platform: Option<String>,
        url: Option<String>,
        hash: Option<String>,
        signature: Option<String>,
        install_path: Option<String>,
        file_size: Option<i32>,
        is_latest: bool,
        status: String,
        pub_date: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: Option<DateTime<Utc>>,
        deleted_at: Option<DateTime<Utc>>,
        arch: Option<String>,
        package_format: Option<String>,
        requires_extract: bool,
        entrypoint_template: Option<String>,
        extract_root: Option<String>,
    }

    const SELECT_AND_ORDER: &str = r#"
        SELECT DISTINCT ON (vt.type_code, v.resource_name)
            vt.type_code,
            v.id, v.type_id, v.resource_name, v.version, v.name, v.notes,
            v.platform, v.url, v.hash, v.signature, v.install_path,
            v.file_size, v.is_latest, v.status, v.pub_date,
            v.created_at, v.updated_at, v.deleted_at,
            v.arch, v.package_format, v.requires_extract, v.entrypoint_template, v.extract_root
        FROM versions v
        INNER JOIN version_types vt ON v.type_id = vt.id
    "#;

    let results: Vec<VersionWithTypeCode> = match (platform, type_code_filter) {
        (Some(plat), Some(tc)) => {
            sqlx::query_as(&format!(
                "{} WHERE vt.type_code = $1 AND vt.is_active = true AND v.platform = $2 \
                 AND v.status = 'active' AND v.deleted_at IS NULL \
                 ORDER BY vt.type_code, v.resource_name, v.is_latest DESC, \
                 COALESCE(v.pub_date, v.created_at) DESC NULLS LAST, v.id DESC",
                SELECT_AND_ORDER
            ))
            .bind(tc)
            .bind(plat)
            .fetch_all(pool)
            .await?
        }
        (Some(plat), None) => {
            sqlx::query_as(&format!(
                "{} WHERE vt.type_code LIKE 'SIMPRINT_KERNEL_%' AND vt.is_active = true \
                 AND v.platform = $1 AND v.status = 'active' AND v.deleted_at IS NULL \
                 ORDER BY vt.type_code, v.resource_name, v.is_latest DESC, \
                 COALESCE(v.pub_date, v.created_at) DESC NULLS LAST, v.id DESC",
                SELECT_AND_ORDER
            ))
            .bind(plat)
            .fetch_all(pool)
            .await?
        }
        (None, Some(tc)) => {
            sqlx::query_as(&format!(
                "{} WHERE vt.type_code = $1 AND vt.is_active = true \
                 AND v.status = 'active' AND v.deleted_at IS NULL \
                 ORDER BY vt.type_code, v.resource_name, v.is_latest DESC, \
                 COALESCE(v.pub_date, v.created_at) DESC NULLS LAST, v.id DESC",
                SELECT_AND_ORDER
            ))
            .bind(tc)
            .fetch_all(pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as(&format!(
                "{} WHERE vt.type_code LIKE 'SIMPRINT_KERNEL_%' AND vt.is_active = true \
                 AND v.status = 'active' AND v.deleted_at IS NULL \
                 ORDER BY vt.type_code, v.resource_name, v.is_latest DESC, \
                 COALESCE(v.pub_date, v.created_at) DESC NULLS LAST, v.id DESC",
                SELECT_AND_ORDER
            ))
            .fetch_all(pool)
            .await?
        }
    };

    let converted: Vec<(String, String, Version)> = results
        .into_iter()
        .map(|r| {
            (
                r.type_code.clone(),
                r.resource_name.clone(),
                Version {
                    id: r.id,
                    type_id: r.type_id,
                    resource_name: r.resource_name,
                    version: r.version,
                    name: r.name.clone(),
                    notes: r.notes,
                    platform: r.platform,
                    url: r.url,
                    hash: r.hash,
                    signature: r.signature,
                    install_path: r.install_path,
                    file_size: r.file_size,
                    is_latest: r.is_latest,
                    status: r.status,
                    pub_date: r.pub_date,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    deleted_at: r.deleted_at,
                    arch: r.arch,
                    package_format: r.package_format,
                    requires_extract: r.requires_extract,
                    entrypoint_template: r.entrypoint_template,
                    extract_root: r.extract_root,
                },
            )
        })
        .collect();

    Ok(converted)
}
