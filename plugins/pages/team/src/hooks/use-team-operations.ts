import { useState } from 'react';
import { addMember as addLocalMember, updateMemberRole, removeMember, batchRemoveMembers } from '../api';
import type { TeamMember } from '../types';

export interface UseTeamOperationsReturn {
  submitting: boolean;
  addMember: (userUuid: string, role: TeamMember['role']) => Promise<{ memberUuid: string }>;
  deleteMember: (memberUuid: string) => Promise<void>;
  batchDeleteMembers: (memberUuids: string[]) => Promise<void>;
  changeMemberRole: (memberUuid: string, newRole: TeamMember['role']) => Promise<TeamMember>;
}

/**
 * 团队成员操作 Hook
 */
export function useTeamOperations(): UseTeamOperationsReturn {
  const [submitting, setSubmitting] = useState(false);

  const addMember = async (userUuid: string, role: TeamMember['role']) => {
    if (!userUuid) {
      throw new Error('请选择本地用户');
    }
    setSubmitting(true);
    try {
      const res = await addLocalMember({ user_uuid: userUuid, role });
      return { memberUuid: res.member_uuid };
    } finally {
      setSubmitting(false);
    }
  };

  // 更新成员角色
  const changeMemberRole = async (memberUuid: string, newRole: TeamMember['role']) => {
    setSubmitting(true);
    try {
      return await updateMemberRole({ member_uuid: memberUuid, role: newRole });
    } finally {
      setSubmitting(false);
    }
  };

  // 删除成员
  const deleteMember = async (memberUuid: string) => {
    setSubmitting(true);
    try {
      await removeMember({ member_uuid: memberUuid });
    } finally {
      setSubmitting(false);
    }
  };

  // 批量删除成员
  const batchDeleteMembers = async (memberUuids: string[]) => {
    setSubmitting(true);
    try {
      await batchRemoveMembers({ member_uuids: memberUuids });
    } finally {
      setSubmitting(false);
    }
  };

  return {
    submitting,
    addMember,
    deleteMember,
    batchDeleteMembers,
    changeMemberRole,
  };
}
