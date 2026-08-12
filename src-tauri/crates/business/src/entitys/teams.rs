use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 创建团队请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTeamRequest {
    pub workspace_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// 更新团队请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateTeamRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_hash: Option<String>,
}

/// 切换团队请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwitchTeamRequest {
    pub team_uuid: Uuid,
}

/// 查询团队成员请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTeamMembersRequest {
    pub workspace_uuid: Uuid,
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<TeamMemberFilters>,
}

/// 团队成员筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TeamMemberFilters {
    pub keyword: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// 邀请成员请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: String,
}

/// 取消邀请请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelInviteRequest {
    pub invitation_uuid: Uuid,
}

/// 更新成员角色请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateMemberRoleRequest {
    pub member_uuid: Uuid,
    pub role: String,
}

/// 更新成员状态请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateMemberStatusRequest {
    pub member_uuid: Uuid,
    pub status: String,
}

/// 移除成员请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveMemberRequest {
    pub member_uuid: Uuid,
}

/// 批量移除成员请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchRemoveMembersRequest {
    pub member_uuids: Vec<Uuid>,
}

/// 接受邀请请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AcceptInvitationRequest {
    pub token: String,
}

/// 拒绝邀请请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RejectInvitationRequest {
    pub token: String,
}

// ========== 响应结构体 ==========

/// 团队列表响应
#[derive(Debug, Clone, Serialize)]
pub struct TeamListResponse {
    pub current_team_uuid: Option<Uuid>,
    pub teams: Vec<TeamItem>,
}

/// 团队列表项
#[derive(Debug, Clone, Serialize)]
pub struct TeamItem {
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub role: String,
    pub members_count: i64,
    pub is_current: bool,
}

/// 团队成员列表响应
#[derive(Debug, Clone, Serialize)]
pub struct MemberListResponse {
    pub items: Vec<crate::dto::TeamMemberDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 接受邀请响应
#[derive(Debug, Clone, Serialize)]
pub struct AcceptInvitationResponse {
    pub team_uuid: Uuid,
}
