use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{entitys::*, models, services, svc_ctx::SvcCtx};

fn payload<T: DeserializeOwned>(data: &Value) -> Result<T, String> {
    serde_json::from_value(data.clone())
        .map_err(|error| format!("Invalid request payload: {error}"))
}

fn value<T: Serialize>(data: T) -> Result<Value, String> {
    serde_json::to_value(data).map_err(|error| format!("Failed to serialize response: {error}"))
}

async fn current_workspace(context: &SvcCtx) -> Result<Uuid, String> {
    models::user::fetch_user_current_workspace(&context.db, context.local_user_uuid)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "No local workspace is selected".to_string())
}

async fn current_team(context: &SvcCtx) -> Result<Uuid, String> {
    services::teams::get_current_team_service(context, context.local_user_uuid)
        .await?
        .ok_or_else(|| "No local team is selected".to_string())
}

/// Dispatch a migrated business POST route locally.
///
/// `None` means the route does not belong to the embedded business service.
pub async fn dispatch_post(
    context: &SvcCtx,
    route: &str,
    data: &Value,
) -> Option<Result<Value, String>> {
    let result = match route.trim_start_matches('/') {
        "browser-kernels/list" => {
            async {
                let platform = data.get("platform").and_then(Value::as_str).unwrap_or("windows");
                let type_code = data
                    .get("type_code")
                    .and_then(Value::as_str)
                    .unwrap_or("SIMPRINT_KERNEL_CHROMIUM");
                value(
                    services::browser_kernels::list_browser_kernels(
                        &context.db,
                        Some(platform),
                        Some(type_code),
                    )
                    .await?,
                )
            }
            .await
        }
        "local-api/get" => {
            async {
                value(
                    services::local_api::get_local_api_config_service(
                        context,
                        context.local_user_uuid,
                    )
                    .await?,
                )
            }
            .await
        }
        "local-api/update" => {
            async {
                let request: UpdateLocalApiConfigRequest = payload(data)?;
                value(
                    services::local_api::update_local_api_config_service(
                        context,
                        context.local_user_uuid,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }
        "local-api/reset-api-key" => {
            async {
                value(
                    services::local_api::reset_local_api_key_service(
                        context,
                        context.local_user_uuid,
                    )
                    .await?,
                )
            }
            .await
        }
        "workspaces/list" => {
            let workspaces =
                services::workspaces::get_user_workspaces_service(context, context.local_user_uuid)
                    .await;
            match workspaces {
                Ok(workspaces) => {
                    let current = current_workspace(context).await.ok();
                    value(WorkspaceListResponse {
                        current_workspace_uuid: current,
                        workspaces: workspaces
                            .into_iter()
                            .map(|workspace| WorkspaceItem {
                                uuid: workspace.uuid,
                                name: workspace.name,
                                workspace_type: workspace.workspace_type,
                                is_current: current == Some(workspace.uuid),
                            })
                            .collect(),
                    })
                }
                Err(error) => Err(error),
            }
        }
        "workspaces/get" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(services::workspaces::get_workspace_service(context, request.uuid).await?)
            }
            .await
        }
        "workspaces/create" => {
            async {
                let request: CreateWorkspaceRequest = payload(data)?;
                let uuid = services::workspaces::create_workspace_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "workspaces/update" => {
            async {
                let request: UpdateWorkspaceRequest = payload(data)?;
                services::workspaces::update_workspace_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "workspaces/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::workspaces::delete_workspace_service(
                    context,
                    context.local_user_uuid,
                    request.uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "workspaces/switch" => {
            async {
                let request: SwitchWorkspaceRequest = payload(data)?;
                services::workspaces::switch_workspace_service(
                    context,
                    request.workspace_uuid,
                    context.local_user_uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }

        "teams/my-teams" => {
            async {
                let teams =
                    services::teams::get_user_teams_service(context, context.local_user_uuid)
                        .await?;
                let current =
                    services::teams::get_current_team_service(context, context.local_user_uuid)
                        .await?;
                let mut items = Vec::with_capacity(teams.len());
                for team in teams {
                    let role = if team.owner_uuid == context.local_user_uuid {
                        "owner".to_string()
                    } else {
                        models::teams::fetch_team_member(
                            &context.db,
                            team.workspace_uuid,
                            team.uuid,
                            context.local_user_uuid,
                        )
                        .await
                        .ok()
                        .flatten()
                        .map(|member| member.role)
                        .unwrap_or_else(|| "member".to_string())
                    };
                    let members_count = models::teams::fetch_team_member_count(
                        &context.db,
                        team.uuid,
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap_or(0);
                    items.push(TeamItem {
                        uuid: team.uuid,
                        name: team.name,
                        description: team.description,
                        role,
                        members_count,
                        is_current: current == Some(team.uuid),
                    });
                }
                value(TeamListResponse {
                    current_team_uuid: current,
                    teams: items,
                })
            }
            .await
        }
        "teams/create" => {
            async {
                let request: CreateTeamRequest = payload(data)?;
                let uuid = services::teams::create_team_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "teams/get" | "teams/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(services::teams::get_team_service(context, request.uuid).await?)
            }
            .await
        }
        "teams/switch" => {
            async {
                let request: SwitchTeamRequest = payload(data)?;
                services::teams::switch_team_service(context, context.local_user_uuid, &request)
                    .await?;
                Ok(Value::Null)
            }
            .await
        }
        "teams/update" => {
            async {
                let request: UpdateTeamRequest = payload(data)?;
                services::teams::update_team_service(
                    context,
                    current_workspace(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "teams/members" => {
            async {
                let request: ListTeamMembersRequest = payload(data)?;
                let (items, total) = services::teams::get_team_members_service(
                    context,
                    current_team(context).await?,
                    &request,
                )
                .await?;
                value(MemberListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "teams/member/add" => {
            async {
                let request: AddMemberRequest = payload(data)?;
                let member_uuid = services::teams::add_member_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(json!({ "member_uuid": member_uuid }))
            }
            .await
        }
        "teams/member/role" => {
            async {
                let request: UpdateMemberRoleRequest = payload(data)?;
                value(
                    services::teams::update_member_role_service(
                        context,
                        current_workspace(context).await?,
                        current_team(context).await?,
                        context.local_user_uuid,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }
        "teams/member/remove" => {
            async {
                let request: RemoveMemberRequest = payload(data)?;
                services::teams::remove_member_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    request.member_uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "teams/leave" => {
            async {
                services::teams::leave_team_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }

        "groups/list" => {
            async {
                value(
                    services::groups::get_groups_service(
                        context,
                        current_workspace(context).await?,
                        current_team(context).await?,
                        1,
                        10_000,
                    )
                    .await?,
                )
            }
            .await
        }
        "groups/create" => {
            async {
                let request: CreateGroupRequest = payload(data)?;
                let uuid = services::groups::create_group_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "groups/update" => {
            async {
                let request: UpdateGroupRequest = payload(data)?;
                services::groups::update_group_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "groups/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::groups::delete_group_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    request.uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "groups/batch-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                for uuid in request.uuids {
                    services::groups::delete_group_service(
                        context,
                        current_workspace(context).await?,
                        current_team(context).await?,
                        context.local_user_uuid,
                        uuid,
                    )
                    .await?;
                }
                Ok(Value::Null)
            }
            .await
        }

        "group-permissions/grant" => {
            async {
                let request: GrantGroupPermissionRequest = payload(data)?;
                services::group_permissions::grant_group_permission_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "group-permissions/revoke" => {
            async {
                let request: RevokeGroupPermissionRequest = payload(data)?;
                services::group_permissions::revoke_group_permission_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "group-permissions/check" => {
            async {
                let request: CheckGroupPermissionRequest = payload(data)?;
                let has_permission = services::group_permissions::check_group_permission_service(
                    context,
                    current_workspace(context).await?,
                    &request,
                )
                .await?;
                value(CheckPermissionResponse {
                    has_permission,
                    permission_type: has_permission.then_some(request.permission_type),
                })
            }
            .await
        }
        "group-permissions/list" => {
            async {
                let request: ListUserGroupPermissionsRequest = payload(data)?;
                let items = services::group_permissions::list_user_group_permissions_service(
                    context, &request,
                )
                .await?;
                value(GroupPermissionListResponse {
                    total: items.len() as i64,
                    items,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }

        "tags/list" => {
            async {
                value(
                    services::tags::get_tags_service(
                        context,
                        context.local_user_uuid,
                        services::teams::get_current_team_service(context, context.local_user_uuid)
                            .await?,
                    )
                    .await?,
                )
            }
            .await
        }
        "tags/create" => {
            async {
                let request: CreateTagRequest = payload(data)?;
                let uuid = services::tags::create_tag_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "tags/update" => {
            async {
                let request: UpdateTagRequest = payload(data)?;
                services::tags::update_tag_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "tags/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::tags::delete_tag_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }

        "proxies/list" => {
            async {
                let request: ListProxiesRequest = payload(data)?;
                let (items, total) = services::proxies::get_proxies_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                value(ProxyListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "proxies/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(services::proxies::get_proxy_service(context, request.uuid).await?)
            }
            .await
        }
        "proxies/create" => {
            async {
                let request: CreateProxyRequest = payload(data)?;
                let uuid = services::proxies::create_proxy_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "proxies/update" => {
            async {
                let request: UpdateProxyRequest = payload(data)?;
                services::proxies::update_proxy_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "proxies/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::proxies::delete_proxy_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }
        "proxies/batch-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                value(
                    services::proxies::batch_delete_proxies_service(context, &request.uuids)
                        .await?,
                )
            }
            .await
        }
        "proxies/batch-import" => {
            async {
                let request: BatchImportProxiesRequest = payload(data)?;
                value(
                    services::proxies::batch_import_proxies_service(
                        context,
                        context.local_user_uuid,
                        current_workspace(context).await?,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }

        "proxy-visibility/set" => {
            async {
                let request: SetProxyVisibleRequest = payload(data)?;
                services::proxy_visibility::set_proxy_visible_to_team_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "proxy-visibility/remove" => {
            async {
                let request: RemoveProxyVisibleRequest = payload(data)?;
                services::proxy_visibility::remove_proxy_visible_from_team_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "proxy-visibility/batch-set" => {
            async {
                let request: BatchSetProxyVisibleRequest = payload(data)?;
                services::proxy_visibility::batch_set_proxy_visible_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "proxy-visibility/list-visible" => {
            async {
                let mut request: ListVisibleProxiesRequest = payload(data)?;
                if request.workspace_uuid.is_nil() {
                    request.workspace_uuid = current_workspace(context).await?;
                }
                value(VisibleProxyListResponse {
                    items: services::proxy_visibility::get_visible_proxies_service(
                        context,
                        context.local_user_uuid,
                        &request,
                    )
                    .await?,
                })
            }
            .await
        }
        "proxy-visibility/list-teams" => {
            async {
                let request: ListProxyVisibleTeamsRequest = payload(data)?;
                value(
                    services::proxy_visibility::get_proxy_visible_teams_service(
                        context,
                        request.proxy_uuid,
                    )
                    .await?,
                )
            }
            .await
        }

        "accounts/list" => {
            async {
                let request: ListAccountsRequest = payload(data)?;
                let (items, total) = services::accounts::get_accounts_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                value(AccountListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "accounts/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(services::accounts::get_account_service(context, request.uuid).await?)
            }
            .await
        }
        "accounts/create" => {
            async {
                let request: CreateAccountRequest = payload(data)?;
                let uuid = services::accounts::create_account_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "accounts/update" => {
            async {
                let request: UpdateAccountRequest = payload(data)?;
                services::accounts::update_account_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "accounts/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::accounts::delete_account_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }
        "accounts/batch-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                value(
                    services::accounts::batch_delete_accounts_service(context, &request.uuids)
                        .await?,
                )
            }
            .await
        }
        "accounts/batch-import" => {
            async {
                let request: BatchImportAccountsRequest = payload(data)?;
                value(
                    services::accounts::batch_import_accounts_service(
                        context,
                        context.local_user_uuid,
                        Some(current_team(context).await?),
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }

        "environments/list" => {
            async {
                let request: ListEnvironmentsRequest = payload(data)?;
                let (items, total) = services::environments::get_environments_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    &request,
                )
                .await?;
                value(EnvironmentListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "environments/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(
                    services::environments::get_environment_detail_service(
                        context,
                        current_workspace(context).await?,
                        current_team(context).await?,
                        context.local_user_uuid,
                        request.uuid,
                    )
                    .await?,
                )
            }
            .await
        }
        "environments/batch-detail" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                let workspace_uuid = current_workspace(context).await?;
                let team_uuid = current_team(context).await?;
                let mut items = std::collections::HashMap::new();
                for uuid in request.uuids {
                    if let Ok(detail) = services::environments::get_environment_detail_service(
                        context,
                        workspace_uuid,
                        team_uuid,
                        context.local_user_uuid,
                        uuid,
                    )
                    .await
                    {
                        items.insert(uuid.to_string(), detail);
                    }
                }
                value(items)
            }
            .await
        }
        "environments/create" => {
            async {
                let request: CreateEnvironmentRequest = payload(data)?;
                let uuid = services::environments::create_environment_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "environments/batch-create" => {
            async {
                let request: BatchCreateEnvironmentRequest = payload(data)?;
                value(
                    services::environments::batch_create_environments_service(
                        context,
                        context.local_user_uuid,
                        current_workspace(context).await?,
                        current_team(context).await?,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }
        "environments/update" => {
            async {
                let request: UpdateEnvironmentRequest = payload(data)?;
                services::environments::update_environment_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::environments::delete_environment_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    request.uuid,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/batch-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                value(
                    services::environments::batch_delete_environments_service(
                        context,
                        &request.uuids,
                    )
                    .await?,
                )
            }
            .await
        }
        "environments/set-proxy" => {
            async {
                let request: SetEnvironmentProxyRequest = payload(data)?;
                services::environments::set_environment_proxy_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/set-accounts" => {
            async {
                let request: SetEnvironmentAccountsRequest = payload(data)?;
                services::accounts::set_environment_accounts_service(
                    context,
                    request.uuid,
                    &request.account_uuids,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/assign-tags" => {
            async {
                let request: AssignTagsRequest = payload(data)?;
                services::environments::assign_tags_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/remove-tag" => {
            async {
                let request: RemoveTagRequest = payload(data)?;
                services::environments::remove_tag_service(context, request.uuid, request.tag_uuid)
                    .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/batch-assign-tags" => {
            async {
                let request: BatchAssignTagRequest = payload(data)?;
                services::environments::batch_assign_tags_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/batch-remove-tags" => {
            async {
                let request: BatchRemoveTagsRequest = payload(data)?;
                services::environments::batch_remove_tags_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/move-to-group" => {
            async {
                let request: MoveToGroupRequest = payload(data)?;
                services::environments::move_to_group_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/batch-move-to-group" => {
            async {
                let request: BatchMoveToGroupRequest = payload(data)?;
                services::environments::batch_move_to_group_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/recycle-bin/list" => {
            async {
                let request: ListEnvironmentsRequest = payload(data)?;
                let (items, total) = services::environments::get_recycle_bin_environments_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                value(EnvironmentListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "environments/recycle-bin/restore" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::environments::restore_environment_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/recycle-bin/batch-restore" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                services::environments::batch_restore_environments_service(context, &request.uuids)
                    .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/recycle-bin/permanent-delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::environments::permanent_delete_environment_service(
                    context,
                    request.uuid,
                    current_workspace(context).await?,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/recycle-bin/batch-permanent-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                services::environments::batch_permanent_delete_environments_service(
                    context,
                    &request.uuids,
                    current_workspace(context).await?,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/urls/list" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(
                    services::environments::get_environment_urls_service(context, request.uuid)
                        .await?,
                )
            }
            .await
        }
        "environments/urls/add" => {
            async {
                let request: AddEnvironmentUrlRequest = payload(data)?;
                let id =
                    services::environments::add_environment_url_service(context, &request).await?;
                Ok(json!({ "id": id }))
            }
            .await
        }
        "environments/urls/delete" => {
            async {
                let request: DeleteEnvironmentUrlRequest = payload(data)?;
                services::environments::delete_environment_url_service(context, request.id).await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/urls/clear" => {
            async {
                let request: ClearEnvironmentUrlsRequest = payload(data)?;
                value(
                    services::environments::clear_environment_urls_service(
                        context,
                        request.environment_uuid,
                    )
                    .await?,
                )
            }
            .await
        }
        "environments/cookies/list" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(
                    services::environments::get_environment_cookies_service(context, request.uuid)
                        .await?,
                )
            }
            .await
        }
        "environments/cookies/add" => {
            async {
                let request: AddEnvironmentCookieRequest = payload(data)?;
                let id = services::environments::add_environment_cookie_service(context, &request)
                    .await?;
                Ok(json!({ "id": id }))
            }
            .await
        }
        "environments/cookies/delete" => {
            async {
                let request: DeleteEnvironmentCookieRequest = payload(data)?;
                services::environments::delete_environment_cookie_service(context, request.id)
                    .await?;
                Ok(Value::Null)
            }
            .await
        }
        "environments/cookies/clear" => {
            async {
                let request: ClearEnvironmentCookiesRequest = payload(data)?;
                value(
                    services::environments::clear_environment_cookies_service(
                        context,
                        request.environment_uuid,
                    )
                    .await?,
                )
            }
            .await
        }

        "templates/list" => {
            async {
                let request: ListTemplatesRequest = payload(data)?;
                let (items, total) = services::templates::get_templates_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    request.is_public,
                    request.pagination.page,
                    request.pagination.page_size,
                )
                .await?;
                value(TemplateListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "templates/detail" => {
            async {
                let request: GetTemplateRequest = payload(data)?;
                value(
                    services::templates::get_template_service(
                        context,
                        request.uuid,
                        request.for_create.unwrap_or(false),
                    )
                    .await?,
                )
            }
            .await
        }
        "templates/create" => {
            async {
                let request: CreateTemplateRequest = payload(data)?;
                let uuid = services::templates::create_template_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "templates/update" => {
            async {
                let request: UpdateTemplateRequest = payload(data)?;
                services::templates::update_template_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "templates/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::templates::delete_template_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }
        "templates/apply" => {
            async {
                let request: ApplyTemplateRequest = payload(data)?;
                services::templates::apply_template_service(
                    context,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "templates/create-from" => {
            async {
                let request: CreateFromTemplateRequest = payload(data)?;
                let uuid = services::templates::create_from_template_service(
                    context,
                    context.local_user_uuid,
                    current_workspace(context).await?,
                    current_team(context).await?,
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }

        "rpa/tasks" => {
            async {
                let request: ListRpaTasksRequest = payload(data)?;
                let (items, total) = services::rpa::get_rpa_tasks_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                value(RpaTaskListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "rpa/tasks/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                let (task, steps, environment_uuids) =
                    services::rpa::get_rpa_task_service(context, request.uuid).await?;
                value(RpaTaskDetailResponse {
                    task,
                    steps,
                    environment_uuids,
                })
            }
            .await
        }
        "rpa/tasks/create" => {
            async {
                let request: CreateRpaTaskRequest = payload(data)?;
                let uuid = services::rpa::create_rpa_task_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }
        "rpa/tasks/update" => {
            async {
                let request: UpdateRpaTaskRequest = payload(data)?;
                services::rpa::update_rpa_task_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }
        "rpa/tasks/delete" => {
            async {
                let request: UuidRequest = payload(data)?;
                services::rpa::delete_rpa_task_service(context, request.uuid).await?;
                Ok(Value::Null)
            }
            .await
        }
        "rpa/tasks/batch-delete" => {
            async {
                let request: BatchUuidRequest = payload(data)?;
                services::rpa::batch_delete_rpa_tasks_service(context, &request.uuids).await?;
                Ok(Value::Null)
            }
            .await
        }
        "rpa/tasks/duplicate" => {
            async {
                let request: DuplicateRpaTaskRequest = payload(data)?;
                let uuid = services::rpa::duplicate_rpa_task_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                Ok(json!({ "uuid": uuid }))
            }
            .await
        }

        "messages/list" => {
            async {
                let request: ListMessagesRequest = payload(data)?;
                value(
                    services::messages::get_user_messages_service(
                        context,
                        context.local_user_uuid,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }
        "messages/read" => {
            async {
                let request: MarkMessageReadRequest = payload(data)?;
                services::messages::mark_message_read_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "messages/batch-read" => {
            async {
                let request: BatchMarkReadRequest = payload(data)?;
                services::messages::batch_mark_messages_read_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "messages/handle" => {
            async {
                let request: HandleMessageRequest = payload(data)?;
                services::messages::handle_message_service(
                    context,
                    context.local_user_uuid,
                    &request,
                )
                .await?;
                Ok(Value::Null)
            }
            .await
        }
        "messages/stats" => {
            async {
                value(
                    services::messages::get_user_message_stats_service(
                        context,
                        context.local_user_uuid,
                    )
                    .await?,
                )
            }
            .await
        }

        "audit/logs" => {
            async {
                let request: ListAuditLogsRequest = payload(data)?;
                let (items, total) = services::audit::get_audit_logs_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                value(AuditLogsListResponse {
                    items,
                    total,
                    page: request.pagination.page,
                    page_size: request.pagination.page_size,
                })
            }
            .await
        }
        "audit/logs/detail" => {
            async {
                let request: UuidRequest = payload(data)?;
                value(services::audit::get_audit_log_service(context, request.uuid).await?)
            }
            .await
        }
        "audit/stats" => {
            async {
                value(
                    services::audit::get_audit_stats_service(
                        context,
                        context.local_user_uuid,
                        Some(current_team(context).await?),
                    )
                    .await?,
                )
            }
            .await
        }
        "audit/logs/export" => {
            async {
                let request: ExportAuditLogsRequest = payload(data)?;
                let (content, filename, mime_type) = services::audit::export_audit_logs_service(
                    context,
                    context.local_user_uuid,
                    Some(current_team(context).await?),
                    &request,
                )
                .await?;
                value(ExportResponse {
                    content,
                    filename,
                    mime_type,
                })
            }
            .await
        }

        "workspace-quotas/get" => {
            async {
                let mut request: GetWorkspaceQuotaRequest = payload(data)?;
                if request.workspace_uuid.is_none() {
                    request.workspace_uuid = Some(current_workspace(context).await?);
                }
                value(
                    services::workspace_quotas::get_workspace_quota_service(context, &request)
                        .await?,
                )
            }
            .await
        }
        "workspace-quotas/update" => {
            async {
                let request: UpdateQuotaUsageRequest = payload(data)?;
                services::workspace_quotas::update_quota_usage_service(context, &request).await?;
                Ok(Value::Null)
            }
            .await
        }

        "preferences/get" => {
            async {
                value(
                    services::preferences::get_preferences_service(
                        context,
                        context.local_user_uuid,
                    )
                    .await?,
                )
            }
            .await
        }
        "preferences/update" => {
            async {
                let request: UpdatePreferencesRequest = payload(data)?;
                value(
                    services::preferences::update_preferences_service(
                        context,
                        context.local_user_uuid,
                        &request,
                    )
                    .await?,
                )
            }
            .await
        }
        _ => return None,
    };

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::DatabaseConfig;

    #[tokio::test]
    async fn migrated_routes_use_the_embedded_database() {
        let mut config = DatabaseConfig::embedded("sqlite::memory:");
        config.max_connections = 1;
        config.min_connections = 1;
        let context = SvcCtx::new(&config).await.expect("context should initialize");

        let workspace_list = dispatch_post(&context, "workspaces/list", &json!({}))
            .await
            .expect("route should be local")
            .expect("workspace list should succeed");
        assert_eq!(workspace_list["workspaces"].as_array().unwrap().len(), 1);

        let kernels = dispatch_post(
            &context,
            "browser-kernels/list",
            &json!({
                "platform": "windows",
                "type_code": "SIMPRINT_KERNEL_CHROMIUM"
            }),
        )
        .await
        .unwrap()
        .expect("browser kernels should be queried from the local registry");
        assert_eq!(
            kernels["SIMPRINT_KERNEL_CHROMIUM"][0]["kernel_id"].as_str().map(str::len),
            Some(64)
        );

        let group = dispatch_post(
            &context,
            "groups/create",
            &json!({ "name": "Local Group", "description": null }),
        )
        .await
        .unwrap()
        .expect("group creation should succeed");
        assert!(group["uuid"].as_str().is_some());

        dispatch_post(
            &context,
            "proxies/create",
            &json!({
                "name": "Local Proxy",
                "host": "127.0.0.1",
                "port": 8080,
                "proxy_type": "http",
                "password": "secret"
            }),
        )
        .await
        .unwrap()
        .expect("proxy creation should succeed");
        let proxies = dispatch_post(
            &context,
            "proxies/list",
            &json!({ "page": 1, "page_size": 20 }),
        )
        .await
        .unwrap()
        .expect("proxy list should succeed");
        assert_eq!(proxies["total"], 1);

        let local_api = dispatch_post(&context, "local-api/get", &json!({}))
            .await
            .unwrap()
            .expect("local API config should be stored locally");
        let api_key = local_api["apiKey"].as_str().unwrap().to_string();
        assert!(api_key.starts_with("sk_local_"));
        dispatch_post(
            &context,
            "local-api/update",
            &json!({ "enabled": true, "port": 18080, "remoteAccess": false }),
        )
        .await
        .unwrap()
        .expect("local API config update should succeed");
        services::local_api::validate_local_api_key_service(
            &context,
            &ValidateLocalApiKeyRequest {
                api_key,
                permission_code: "workspaces.list".to_string(),
            },
        )
        .await
        .expect("local API key should validate without a remote cache");

        assert!(dispatch_post(&context, "auth/login", &json!({})).await.is_none());
    }
}
