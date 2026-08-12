/**
 * 团队管理 API 类型定义
 */

// 从 types 重新导出，保持类型一致
export type { TeamMember, TeamRole, TeamStatus, RoleFilter, Team } from '../types';

// ============ 请求类型 ============

export interface ListTeamMembersRequest {
  workspace_uuid: string;
  page?: number;
  page_size?: number;
  filters?: {
    keyword?: string;
    role?: string;
    status?: string;
  };
}

export interface AddMemberRequest {
  user_uuid: string;
  role: string;
}

export interface InviteResponse {
  member_uuid: string;
}

export interface UpdateMemberRoleRequest {
  member_uuid: string;
  role: string;
}

export interface RemoveMemberRequest {
  member_uuid: string;
}

export interface BatchRemoveMembersRequest {
  member_uuids: string[];
}

export interface SwitchTeamRequest {
  team_uuid: string;
}

export interface GetTeamRequest {
  uuid: string;
}

export interface CreateTeamRequest {
  workspace_uuid: string;
  name: string;
  description?: string;
}

export interface UpdateTeamRequest {
  uuid: string;
  name?: string;
  description?: string;
  avatar_hash?: string;
}

// ============ 响应类型 ============

export interface TeamMemberListResponse {
  items: TeamMember[];
  total: number;
  page: number;
  page_size: number;
}

export interface TeamMemberDto {
  id: number;
  team_uuid: string;
  user_uuid: string;
  role: string;
  joined_at: string;
  invited_by?: string;
  environment_count: number;
  group_count: number;
  status: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
  // 从 user_infos 表关联的用户信息（后端已关联查询）
  name?: string;
  email?: string;
  avatar?: string;
}

export interface TeamDto {
  id: number;
  uuid: string;
  name: string;
  description?: string;
  owner_uuid: string;
  avatar_hash?: string;
  max_members: number;
  max_environments: number;
  max_proxies: number;
  default_proxy_uuid?: string;
  status: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface TeamItem {
  uuid: string;
  name: string;
  description?: string;
  role: string;
  members_count: number;
  is_current: boolean;
}

export interface TeamListResponse {
  current_team_uuid?: string;
  teams: TeamItem[];
}

export interface CreateResponse {
  uuid: string;
}
