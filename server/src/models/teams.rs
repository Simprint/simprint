use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{LoginHistoryDto, TeamDto, TeamInvitationDto, TeamMemberDto};
use crate::entitys::CreateTeamRequest;

// ============ Teams ============

/// 创建团队
pub async fn insert_team(
    pool: &Pool<Postgres>,
    owner_uuid: Uuid,
    payload: &CreateTeamRequest,
) -> Result<Uuid, Error> {
    let mut tx = pool.begin().await?;

    // 1. 创建团队
    let team_uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO teams (workspace_uuid, name, description, owner_uuid)
        VALUES ($1, $2, $3, $4)
        RETURNING uuid;
        "#,
    )
    .bind(payload.workspace_uuid)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(owner_uuid)
    .fetch_one(&mut *tx)
    .await?;

    // 2. 添加所有者为成员
    sqlx::query(
        r#"
        INSERT INTO team_members (team_uuid, workspace_uuid, user_uuid, role, status)
        VALUES ($1, $2, $3, 'owner', 'active');
        "#,
    )
    .bind(team_uuid)
    .bind(payload.workspace_uuid)
    .bind(owner_uuid)
    .execute(&mut *tx)
    .await?;

    // 3. 设置为用户当前团队
    sqlx::query(
        r#"
        UPDATE user_infos SET current_team_uuid = $1 WHERE user_uuid = $2;
        "#,
    )
    .bind(team_uuid)
    .bind(owner_uuid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(team_uuid)
}

/// 根据 UUID 查询团队
pub async fn fetch_team_by_uuid(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
) -> Result<Option<TeamDto>, Error> {
    let rec = sqlx::query_as::<_, TeamDto>(
        r#"
        SELECT id, uuid, workspace_uuid, name, description, owner_uuid, avatar_hash,
               status, created_at, updated_at, deleted_at
        FROM teams
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询用户在工作空间中所属的所有团队（工作空间级别）
pub async fn fetch_user_teams(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<Vec<TeamDto>, Error> {
    let recs = sqlx::query_as::<_, TeamDto>(
        r#"
        SELECT t.id, t.uuid, t.workspace_uuid, t.name, t.description, t.owner_uuid, t.avatar_hash,
               t.status, t.created_at, t.updated_at, t.deleted_at
        FROM teams t
        INNER JOIN team_members tm ON t.uuid = tm.team_uuid
        WHERE tm.workspace_uuid = $1 AND tm.user_uuid = $2 AND t.deleted_at IS NULL AND tm.deleted_at IS NULL
        ORDER BY t.created_at
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询用户当前团队
pub async fn fetch_user_current_team(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<Uuid>, Error> {
    let rec: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT current_team_uuid FROM user_infos
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 设置用户当前团队
pub async fn set_user_current_team(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos SET current_team_uuid = $1 WHERE user_uuid = $2
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 清除用户当前团队
pub async fn clear_user_current_team(pool: &Pool<Postgres>, user_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos SET current_team_uuid = NULL WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新团队信息
pub async fn update_team(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    avatar_hash: Option<&str>,
) -> Result<(), Error> {
    let mut query = String::from("UPDATE teams SET updated_at = NOW()");
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![];

    if let Some(n) = name {
        query.push_str(", name = $");
        params.push(Box::new(n));
        query.push_str(&format!("{}", params.len()));
    }

    if let Some(d) = description {
        query.push_str(", description = $");
        params.push(Box::new(d));
        query.push_str(&format!("{}", params.len()));
    }

    if let Some(a) = avatar_hash {
        query.push_str(", avatar_hash = $");
        params.push(Box::new(a));
        query.push_str(&format!("{}", params.len()));
    }

    query.push_str(" WHERE uuid = $");
    params.push(Box::new(team_uuid));
    query.push_str(&format!("{}", params.len()));

    // 这里需要动态构建查询，但 sqlx 不支持动态查询，所以使用条件分支
    if name.is_some()
        && description.is_some()
        && avatar_hash.is_some()
    {
        sqlx::query(
            r#"
        UPDATE teams
            SET name = $1, description = $2, avatar_hash = $3, updated_at = NOW()
            WHERE uuid = $4
        "#,
        )
        .bind(name.unwrap())
        .bind(description.unwrap())
        .bind(avatar_hash.unwrap())
        .bind(team_uuid)
        .execute(pool)
        .await?;
    } else if name.is_some() && description.is_some() {
        sqlx::query(
            r#"
        UPDATE teams
            SET name = $1, description = $2, updated_at = NOW()
            WHERE uuid = $3
        "#,
        )
        .bind(name.unwrap())
        .bind(description.unwrap())
        .bind(team_uuid)
        .execute(pool)
        .await?;
    } else if name.is_some() {
        sqlx::query(
            r#"
        UPDATE teams
            SET name = $1, updated_at = NOW()
            WHERE uuid = $2
        "#,
        )
        .bind(name.unwrap())
        .bind(team_uuid)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ============ Team Members ============

/// 获取团队成员数量
pub async fn fetch_team_member_count(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    keyword: Option<&str>,
    role: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let mut query = String::from(
        r#"
        SELECT COUNT(*) FROM team_members tm
        LEFT JOIN user_infos ui ON tm.user_uuid = ui.user_uuid
        WHERE tm.team_uuid = $1 AND tm.deleted_at IS NULL
        "#,
    );

    let mut param_index = 2;

    // 默认只查询 active 状态
    if status.is_some() {
        query.push_str(&format!(" AND tm.status = ${}", param_index));
        param_index += 1;
    } else {
        query.push_str(" AND tm.status = 'active'");
    }

    if role.is_some() {
        query.push_str(&format!(" AND tm.role = ${}", param_index));
        param_index += 1;
    }

    if keyword.is_some() {
        query.push_str(&format!(
            " AND (ui.nickname ILIKE ${} OR ui.email ILIKE ${})",
            param_index, param_index
        ));
    }

    let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(team_uuid);

    if let Some(s) = status {
        query_builder = query_builder.bind(s);
    }

    if let Some(r) = role {
        query_builder = query_builder.bind(r);
    }

    if let Some(k) = keyword {
        let keyword_pattern = format!("%{}%", k);
        query_builder = query_builder.bind(keyword_pattern);
    }

    let count = query_builder.fetch_one(pool).await?;

    Ok(count)
}

/// 查询团队成员列表（关联用户信息，支持筛选）
pub async fn fetch_team_members(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    offset: i64,
    limit: i64,
    keyword: Option<&str>,
    role: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<TeamMemberDto>, Error> {
    let mut query = String::from(
        r#"
        SELECT 
            tm.id, 
            tm.team_uuid,
            tm.workspace_uuid,
            tm.user_uuid, 
            tm.role, 
            tm.joined_at, 
            tm.invited_by,
            tm.status, 
            tm.created_at, 
            tm.updated_at, 
            tm.deleted_at,
            ui.nickname AS name,
            ui.email AS email,
            ui.avatar_hash AS avatar
        FROM team_members tm
        LEFT JOIN user_infos ui ON tm.user_uuid = ui.user_uuid
        WHERE tm.team_uuid = $1 AND tm.deleted_at IS NULL
        "#,
    );

    let mut param_index = 2;

    // 默认只查询 active 状态
    if status.is_some() {
        query.push_str(&format!(" AND tm.status = ${}", param_index));
        param_index += 1;
    } else {
        query.push_str(" AND tm.status = 'active'");
    }

    if role.is_some() {
        query.push_str(&format!(" AND tm.role = ${}", param_index));
        param_index += 1;
    }

    if keyword.is_some() {
        query.push_str(&format!(
            " AND (ui.nickname ILIKE ${} OR ui.email ILIKE ${})",
            param_index, param_index
        ));
        param_index += 1;
    }

    query.push_str(&format!(
        r#"
        ORDER BY
            CASE tm.role
                WHEN 'owner' THEN 1
                WHEN 'admin' THEN 2
                WHEN 'editor' THEN 3
                ELSE 4
            END,
            tm.joined_at
        LIMIT ${} OFFSET ${}
        "#,
        param_index,
        param_index + 1
    ));

    let mut query_builder = sqlx::query_as::<_, TeamMemberDto>(&query).bind(team_uuid);

    if let Some(s) = status {
        query_builder = query_builder.bind(s);
    }

    if let Some(r) = role {
        query_builder = query_builder.bind(r);
    }

    if let Some(k) = keyword {
        let keyword_pattern = format!("%{}%", k);
        query_builder = query_builder.bind(keyword_pattern);
    }

    query_builder = query_builder.bind(limit).bind(offset);

    let recs = query_builder.fetch_all(pool).await?;

    Ok(recs)
}

/// 查询用户在团队中的成员信息（工作空间级别）
pub async fn fetch_team_member(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<Option<TeamMemberDto>, Error> {
    let rec = sqlx::query_as::<_, TeamMemberDto>(
        r#"
        SELECT 
            tm.id, 
            tm.team_uuid,
            tm.workspace_uuid,
            tm.user_uuid, 
            tm.role, 
            tm.joined_at, 
            tm.invited_by,
            tm.status, 
            tm.created_at, 
            tm.updated_at, 
            tm.deleted_at,
            ui.nickname AS name,
            ui.email AS email,
            ui.avatar_hash AS avatar
        FROM team_members tm
        LEFT JOIN user_infos ui ON tm.user_uuid = ui.user_uuid
        WHERE tm.workspace_uuid = $1 AND tm.team_uuid = $2 AND tm.user_uuid = $3 AND tm.deleted_at IS NULL
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 添加团队成员
pub async fn insert_team_member(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    user_uuid: Uuid,
    role: &str,
    invited_by: Option<Uuid>,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO team_members (team_uuid, workspace_uuid, user_uuid, role, invited_by, status)
        VALUES ($1, (SELECT workspace_uuid FROM teams WHERE uuid = $1), $2, $3, $4, 'active')
        RETURNING id;
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(role)
    .bind(invited_by)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 更新成员角色
pub async fn update_member_role(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    user_uuid: Uuid,
    role: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_members
        SET role = $1, updated_at = NOW()
        WHERE team_uuid = $2 AND user_uuid = $3 AND deleted_at IS NULL
        "#,
    )
    .bind(role)
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新成员状态
pub async fn update_member_status(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    user_uuid: Uuid,
    status: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_members
        SET status = $1
        WHERE team_uuid = $2 AND user_uuid = $3 AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 移除成员（软删除）
pub async fn remove_team_member(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_members
        SET deleted_at = NOW(), status = 'inactive'
        WHERE team_uuid = $1 AND user_uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Team Invitations ============

/// 创建团队邀请
pub async fn insert_team_invitation(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    email: &str,
    role: &str,
    invited_by: Uuid,
    token: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO team_invitations (team_uuid, email, role, invited_by, token, expires_at, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        RETURNING uuid;
        "#,
    )
    .bind(team_uuid)
    .bind(email)
    .bind(role)
    .bind(invited_by)
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询团队邀请
pub async fn fetch_team_invitation_by_token(
    pool: &Pool<Postgres>,
    token: &str,
) -> Result<Option<TeamInvitationDto>, Error> {
    let rec = sqlx::query_as::<_, TeamInvitationDto>(
        r#"
        SELECT id, uuid, team_uuid, email, role, invited_by, token, expires_at, status, accepted_at, created_at, updated_at
        FROM team_invitations
        WHERE token = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询团队的待处理邀请
pub async fn fetch_pending_invitations(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
) -> Result<Vec<TeamInvitationDto>, Error> {
    let recs = sqlx::query_as::<_, TeamInvitationDto>(
        r#"
        SELECT id, uuid, team_uuid, email, role, invited_by, token, expires_at, status, accepted_at, created_at, updated_at
        FROM team_invitations
        WHERE team_uuid = $1 AND status = 'pending' AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 检查是否有待处理的邀请
pub async fn has_pending_invitation(
    pool: &Pool<Postgres>,
    team_uuid: Uuid,
    email: &str,
) -> Result<bool, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM team_invitations
        WHERE team_uuid = $1 AND email = $2 AND status = 'pending' AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .bind(email)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// 更新邀请状态
pub async fn update_invitation_status(
    pool: &Pool<Postgres>,
    invitation_uuid: Uuid,
    status: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_invitations
        SET status = $1, updated_at = NOW()
        WHERE uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(invitation_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询邀请（通过 token，别名函数）
pub async fn fetch_invitation_by_token(
    pool: &Pool<Postgres>,
    token: &str,
) -> Result<Option<TeamInvitationDto>, Error> {
    fetch_team_invitation_by_token(pool, token).await
}

/// 取消邀请
pub async fn cancel_team_invitation(
    pool: &Pool<Postgres>,
    invitation_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE team_invitations
        SET status = 'cancelled', deleted_at = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(invitation_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 接受邀请
pub async fn accept_team_invitation(
    pool: &Pool<Postgres>,
    invitation_uuid: Uuid,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
) -> Result<Uuid, Error> {
    let mut tx = pool.begin().await?;

    // 1. 获取邀请信息
    let invitation = sqlx::query_as::<_, TeamInvitationDto>(
        r#"
        SELECT id, uuid, team_uuid, email, role, invited_by, token, expires_at, status, accepted_at, created_at, updated_at
        FROM team_invitations
        WHERE uuid = $1 AND deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(invitation_uuid)
    .fetch_optional(&mut *tx)
    .await?;

    let invitation = invitation.ok_or_else(|| Error::RowNotFound)?;

    // 2. 检查邀请状态
    if invitation.status != "pending" {
        return Err(Error::RowNotFound);
    }

    // 3. 检查是否过期
    if invitation.expires_at < chrono::Utc::now() {
        return Err(Error::RowNotFound);
    }

    // 4. 添加成员（如果已存在则更新状态为 active）
    sqlx::query(
        r#"
        INSERT INTO team_members (team_uuid, workspace_uuid, user_uuid, role, invited_by, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        ON CONFLICT (team_uuid, user_uuid, workspace_uuid) DO UPDATE SET
            role = EXCLUDED.role,
            invited_by = EXCLUDED.invited_by,
            status = 'active',
            deleted_at = NULL,
            updated_at = NOW()
        "#,
    )
    .bind(invitation.team_uuid)
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(&invitation.role)
    .bind(invitation.invited_by)
    .execute(&mut *tx)
    .await?;

    // 5. 更新邀请状态
    sqlx::query(
        r#"
        UPDATE team_invitations
        SET status = 'accepted', accepted_at = NOW()
        WHERE uuid = $1
        "#,
    )
    .bind(invitation_uuid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(invitation.team_uuid)
}

/// 拒绝邀请
pub async fn reject_team_invitation(
    pool: &Pool<Postgres>,
    invitation_uuid: Uuid,
    _user_uuid: Uuid,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    // 1. 获取邀请信息
    let invitation = sqlx::query_as::<_, TeamInvitationDto>(
        r#"
        SELECT id, uuid, team_uuid, email, role, invited_by, token, expires_at, status, accepted_at, created_at, updated_at
        FROM team_invitations
        WHERE uuid = $1 AND deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(invitation_uuid)
    .fetch_optional(&mut *tx)
    .await?;

    let invitation = invitation.ok_or_else(|| Error::RowNotFound)?;

    // 2. 检查邀请状态（只有 pending 状态的邀请才能被拒绝）
    if invitation.status != "pending" {
        return Err(Error::RowNotFound);
    }

    // 3. 检查是否过期（过期的邀请也可以标记为拒绝）
    // 这里不检查过期时间，允许拒绝已过期的邀请

    // 4. 更新邀请状态为 rejected
    sqlx::query(
        r#"
        UPDATE team_invitations
        SET status = 'rejected', updated_at = NOW()
        WHERE uuid = $1
        "#,
    )
    .bind(invitation_uuid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

// ============ Login History ============

/// 记录登录历史
pub async fn insert_login_history(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    ip_address: &str,
    device_info: Option<&str>,
    user_agent: Option<&str>,
    location: Option<&str>,
    country: Option<&str>,
    city: Option<&str>,
    success: bool,
    failure_reason: Option<&str>,
) -> Result<i64, Error> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO login_history (user_uuid, ip_address, device_info, user_agent, location, country, city, success, failure_reason)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(ip_address)
    .bind(device_info)
    .bind(user_agent)
    .bind(location)
    .bind(country)
    .bind(city)
    .bind(success)
    .bind(failure_reason)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 查询用户登录历史
pub async fn fetch_user_login_history(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    limit: i64,
) -> Result<Vec<LoginHistoryDto>, Error> {
    let recs = sqlx::query_as::<_, LoginHistoryDto>(
        r#"
        SELECT id, user_uuid, ip_address, device_info, user_agent, location, country, city, success, failure_reason, created_at
        FROM login_history
        WHERE user_uuid = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}
