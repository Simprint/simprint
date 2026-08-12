import { create } from 'zustand';
import type { User } from '../../types/store.types';
import type { AuthActions, AuthState } from './auth-store.types';

interface LocalUserResponse {
  uuid: string;
  nickname: string;
  avatar: string;
  hasPassword: boolean;
  currentWorkspaceUuid?: string | null;
  currentTeamUuid?: string | null;
}

function mapLocalUser(user: LocalUserResponse): User {
  return {
    uuid: user.uuid,
    id: user.uuid,
    nickname: user.nickname,
    avatar: user.avatar,
    has_password: user.hasPassword,
    status: 'active',
    current_workspace_uuid: user.currentWorkspaceUuid ?? null,
    current_team_uuid: user.currentTeamUuid ?? null,
  };
}

export const useAuthStore = create<AuthState & AuthActions>((set, get) => ({
  user: null,
  isAuthenticated: false,
  isInitializing: false,
  currentWorkspaceUuid: null,
  currentTeamUuid: null,

  setUser: (user: User) =>
    set({
      user,
      isAuthenticated: true,
      currentWorkspaceUuid: user.current_workspace_uuid || null,
      currentTeamUuid: user.current_team_uuid || null,
    }),

  clearUser: () =>
    set({
      user: null,
      isAuthenticated: false,
      currentWorkspaceUuid: null,
      currentTeamUuid: null,
    }),

  setCurrentWorkspace: (workspaceUuid: string | null) =>
    set({
      currentWorkspaceUuid: workspaceUuid,
      user: get().user
        ? { ...get().user!, current_workspace_uuid: workspaceUuid }
        : null,
    }),

  setCurrentTeam: (teamUuid: string | null) =>
    set({
      currentTeamUuid: teamUuid,
      user: get().user ? { ...get().user!, current_team_uuid: teamUuid } : null,
    }),

  initAuth: async () => {
    if (get().isInitializing || get().user) return;
    set({ isInitializing: true });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const currentUser = await invoke<LocalUserResponse | null>('get_current_local_user');
      if (currentUser) {
        const user = mapLocalUser(currentUser);
        set({
          user,
          isAuthenticated: true,
          isInitializing: false,
          currentWorkspaceUuid: user.current_workspace_uuid || null,
          currentTeamUuid: user.current_team_uuid || null,
        });
        return;
      }
    } catch (error) {
      console.error('[AuthStore] 初始化本地用户会话失败:', error);
    }
    set({ user: null, isAuthenticated: false, isInitializing: false });
  },
}));
