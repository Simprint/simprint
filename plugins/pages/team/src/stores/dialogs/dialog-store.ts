import { create } from 'zustand';
import type { TeamMember, Team } from '../../types';

interface DialogState {
  // 邀请成员对话框
  inviteDialogOpen: boolean;
  inviteUserUuid: string;
  inviteRole: TeamMember['role'];

  // 删除成员对话框
  deleteDialogOpen: boolean;
  deletingMember: { id: string; name: string } | null;

  // 批量删除对话框
  batchDeleteDialogOpen: boolean;

  // 角色变更对话框
  roleChangeDialogOpen: boolean;
  changingMember: TeamMember | null;
  newRole: TeamMember['role'];

  // 切换团队对话框
  switchTeamDialogOpen: boolean;
  selectedTeamId: string;
  currentTeam: Team | null;
}

interface DialogActions {
  // 邀请成员对话框
  setInviteDialogOpen: (open: boolean) => void;
  setInviteUserUuid: (userUuid: string) => void;
  setInviteRole: (role: TeamMember['role']) => void;
  openInviteDialog: () => void;
  closeInviteDialog: () => void;

  // 删除成员对话框
  setDeleteDialogOpen: (open: boolean) => void;
  setDeletingMember: (member: { id: string; name: string } | null) => void;
  openDeleteDialog: (member: { id: string; name: string }) => void;
  closeDeleteDialog: () => void;

  // 批量删除对话框
  setBatchDeleteDialogOpen: (open: boolean) => void;
  openBatchDeleteDialog: () => void;
  closeBatchDeleteDialog: () => void;

  // 角色变更对话框
  setRoleChangeDialogOpen: (open: boolean) => void;
  setChangingMember: (member: TeamMember | null) => void;
  setNewRole: (role: TeamMember['role']) => void;
  openRoleChangeDialog: (member: TeamMember, newRole: TeamMember['role']) => void;
  closeRoleChangeDialog: () => void;

  // 切换团队对话框
  setSwitchTeamDialogOpen: (open: boolean) => void;
  setSelectedTeamId: (teamId: string) => void;
  setCurrentTeam: (team: Team | null) => void;
  openSwitchTeamDialog: () => void;
  closeSwitchTeamDialog: () => void;
}

/**
 * 团队对话框状态 Store
 * 管理所有对话框的开关状态和表单数据
 */
export const useTeamDialogStore = create<DialogState & DialogActions>((set) => ({
  // 初始状态
  inviteDialogOpen: false,
  inviteUserUuid: '',
  inviteRole: 'viewer',
  deleteDialogOpen: false,
  deletingMember: null,
  batchDeleteDialogOpen: false,
  roleChangeDialogOpen: false,
  changingMember: null,
  newRole: 'viewer',
  switchTeamDialogOpen: false,
  selectedTeamId: '',
  currentTeam: null,

  // 邀请成员对话框
  setInviteDialogOpen: (open) => set({ inviteDialogOpen: open }),
  setInviteUserUuid: (userUuid) => set({ inviteUserUuid: userUuid }),
  setInviteRole: (role) => set({ inviteRole: role }),
  openInviteDialog: () => set({ inviteDialogOpen: true, inviteUserUuid: '', inviteRole: 'viewer' }),
  closeInviteDialog: () => set({ inviteDialogOpen: false, inviteUserUuid: '', inviteRole: 'viewer' }),

  // 删除成员对话框
  setDeleteDialogOpen: (open) => set({ deleteDialogOpen: open }),
  setDeletingMember: (member) => set({ deletingMember: member }),
  openDeleteDialog: (member) => set({ deleteDialogOpen: true, deletingMember: member }),
  closeDeleteDialog: () => set({ deleteDialogOpen: false, deletingMember: null }),

  // 批量删除对话框
  setBatchDeleteDialogOpen: (open) => set({ batchDeleteDialogOpen: open }),
  openBatchDeleteDialog: () => set({ batchDeleteDialogOpen: true }),
  closeBatchDeleteDialog: () => set({ batchDeleteDialogOpen: false }),

  // 角色变更对话框
  setRoleChangeDialogOpen: (open) => set({ roleChangeDialogOpen: open }),
  setChangingMember: (member) => set({ changingMember: member }),
  setNewRole: (role) => set({ newRole: role }),
  openRoleChangeDialog: (member, newRole) =>
    set({ roleChangeDialogOpen: true, changingMember: member, newRole }),
  closeRoleChangeDialog: () =>
    set({ roleChangeDialogOpen: false, changingMember: null, newRole: 'viewer' }),

  // 切换团队对话框
  setSwitchTeamDialogOpen: (open) => set({ switchTeamDialogOpen: open }),
  setSelectedTeamId: (teamId) => set({ selectedTeamId: teamId }),
  setCurrentTeam: (team) => set({ currentTeam: team }),
  openSwitchTeamDialog: () => set({ switchTeamDialogOpen: true, selectedTeamId: '' }),
  closeSwitchTeamDialog: () => set({ switchTeamDialogOpen: false, selectedTeamId: '' }),
}));
