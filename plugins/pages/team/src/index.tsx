import { extensionRegistry } from '@slotkitjs/core';
import { useTranslation } from 'react-i18next';
import { teamResources } from './i18n/resources';
import { TeamHeader } from './components/team-header';
import { TeamStats } from './components/team-stats';
import { TeamTable } from './components/team-table';
import { TeamBatchActions } from './components/team-batch-actions';
import { TeamPagination } from './components/team-pagination';
import { TeamInviteDialog } from './components/team-invite-dialog';
import { TeamDeleteDialog } from './components/team-delete-dialog';
import { TeamBatchDeleteDialog } from './components/team-batch-delete-dialog';
import { TeamRoleChangeDialog } from './components/team-role-change-dialog';
import { useTeamMembers } from './hooks/use-team-members';
import { useTeamPagination } from './hooks/use-team-pagination';
import { useTeamStats } from './hooks/use-team-stats';
import { useTeamOperations } from './hooks/use-team-operations';
import { useTeamHandlers } from './hooks/use-team-handlers';
import { useTeamFiltersStore, useTeamSelectionStore, useTeamDialogStore } from './stores';

const TeamPage: React.FC = () => {
  useTranslation('team'); // 注册 i18n namespace

  // Stores
  const filtersStore = useTeamFiltersStore();
  const selectionStore = useTeamSelectionStore();
  const dialogStore = useTeamDialogStore();

  // 数据获取
  const { members, total, loading, error, refresh } = useTeamMembers();

  // 分页
  const { paginatedMembers, totalPages, startIndex } = useTeamPagination({
    members,
    total,
  });

  // 统计
  const stats = useTeamStats({ members });

  // 操作
  const operations = useTeamOperations();

  // 事件处理
  const handlers = useTeamHandlers({
    operations,
    onRefresh: refresh,
  });

  // 当分页变化时，清除选择
  const handlePageChange = (page: number) => {
    filtersStore.setCurrentPage(page);
    selectionStore.clearSelection();
  };

  return (
    <div className="flex flex-col h-[calc(100vh-50px)] relative">
      <TeamHeader onTeamSwitched={refresh} />

      <TeamStats
        total={stats.total}
        active={stats.active}
        pending={stats.pending}
        totalEnvironments={stats.totalEnvironments}
        onRefresh={refresh}
      />

      {error && <div className="px-6 py-2 text-xs text-destructive">团队成员加载失败：{error}</div>}

      <TeamTable
        members={paginatedMembers}
        onChangeRole={handlers.handleChangeRole}
        onDelete={handlers.handleDeleteMember}
        loading={loading}
        startIndex={startIndex}
      />

      <TeamPagination
        currentPage={filtersStore.currentPage}
        totalPages={totalPages}
        onPageChange={handlePageChange}
      />

      <TeamBatchActions
        selectedCount={selectionStore.selectedIds.size}
        onClear={selectionStore.clearSelection}
        onDelete={() => handlers.handleBatchDelete(selectionStore.selectedIds)}
      />

      {/* 邀请成员对话框 */}
      <TeamInviteDialog
        open={dialogStore.inviteDialogOpen}
        userUuid={dialogStore.inviteUserUuid}
        role={dialogStore.inviteRole}
        submitting={operations.submitting}
        onOpenChange={dialogStore.setInviteDialogOpen}
        onUserChange={dialogStore.setInviteUserUuid}
        onRoleChange={dialogStore.setInviteRole}
        onSubmit={handlers.handleSubmitInvite}
      />

      {/* 删除成员确认对话框 */}
      <TeamDeleteDialog
        open={dialogStore.deleteDialogOpen}
        memberName={dialogStore.deletingMember?.name || null}
        onOpenChange={dialogStore.setDeleteDialogOpen}
        onConfirm={handlers.handleConfirmDelete}
      />

      {/* 批量删除确认对话框 */}
      <TeamBatchDeleteDialog
        open={dialogStore.batchDeleteDialogOpen}
        count={selectionStore.selectedIds.size}
        onOpenChange={dialogStore.setBatchDeleteDialogOpen}
        onConfirm={() => handlers.handleConfirmBatchDelete(selectionStore.selectedIds)}
      />

      {/* 角色变更确认对话框 */}
      <TeamRoleChangeDialog
        open={dialogStore.roleChangeDialogOpen}
        member={dialogStore.changingMember}
        newRole={dialogStore.newRole}
        submitting={operations.submitting}
        onOpenChange={dialogStore.setRoleChangeDialogOpen}
        onConfirm={handlers.handleConfirmRoleChange}
      />
    </div>
  );
};

// 在模块加载时贡献路由
try {
  extensionRegistry.contribute('routes', {
    contributorId: 'team',
    value: {
      path: '/team',
      Component: TeamPage,
    },
    priority: 10,
  });
  console.log('[team] Route contributed at module load: /team');
} catch (error) {
  console.warn('[team] Failed to contribute route at module load:', error);
}

try {
  extensionRegistry.contribute('i18n:resources', {
    contributorId: 'team',
    value: {
      namespace: 'team',
      resources: teamResources,
    },
    priority: 10,
  });
} catch (error) {
  console.warn('[team] Failed to contribute i18n resources:', error);
}

const teamPlugin = {
  id: 'team',
  name: 'Team Management',
  version: '1.0.0',
  component: TeamPage,
  slots: [],
};

export default teamPlugin;
