import { toast } from 'sonner';
import { useTeamDialogStore } from '../stores';
import type { TeamMember } from '../types';
import type { UseTeamOperationsReturn } from './use-team-operations';

interface UseTeamHandlersParams {
  operations: UseTeamOperationsReturn;
  onRefresh: () => Promise<void>;
}

/**
 * 团队成员事件处理 Hook
 */
export function useTeamHandlers({ operations, onRefresh }: UseTeamHandlersParams) {
  const dialogStore = useTeamDialogStore();

  // 添加本地成员
  const handleInvite = () => {
    dialogStore.openInviteDialog();
  };

  const handleSubmitInvite = async () => {
    if (!dialogStore.inviteUserUuid) {
      toast.warning('请选择本地用户');
      return;
    }
    try {
      await operations.addMember(dialogStore.inviteUserUuid, dialogStore.inviteRole);
      dialogStore.closeInviteDialog();
      toast.success('本地用户已加入团队');
      await onRefresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '添加成员失败');
    }
  };

  // 删除成员
  const handleDeleteMember = (id: string, name: string) => {
    dialogStore.openDeleteDialog({ id, name });
  };

  const handleConfirmDelete = async () => {
    if (!dialogStore.deletingMember) return;
    try {
      await operations.deleteMember(dialogStore.deletingMember.id);
      dialogStore.closeDeleteDialog();
      toast.success('移除成员成功');
      await onRefresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '移除成员失败');
    }
  };

  // 批量删除
  const handleBatchDelete = (selectedIds: Set<string>) => {
    if (selectedIds.size === 0) return;
    dialogStore.openBatchDeleteDialog();
  };

  const handleConfirmBatchDelete = async (selectedIds: Set<string>) => {
    try {
      await operations.batchDeleteMembers(Array.from(selectedIds));
      dialogStore.closeBatchDeleteDialog();
      toast.success(`成功移除 ${selectedIds.size} 名成员`);
      await onRefresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '批量移除失败');
    }
  };

  // 更改角色
  const handleChangeRole = (member: TeamMember, newRole: TeamMember['role']) => {
    dialogStore.openRoleChangeDialog(member, newRole);
  };

  const handleConfirmRoleChange = async () => {
    if (!dialogStore.changingMember) return;
    try {
      await operations.changeMemberRole(dialogStore.changingMember.id, dialogStore.newRole);
      dialogStore.closeRoleChangeDialog();
      toast.success('更新角色成功');
      await onRefresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '更新角色失败');
    }
  };

  return {
    handleInvite,
    handleSubmitInvite,
    handleDeleteMember,
    handleConfirmDelete,
    handleBatchDelete,
    handleConfirmBatchDelete,
    handleChangeRole,
    handleConfirmRoleChange,
  };
}
