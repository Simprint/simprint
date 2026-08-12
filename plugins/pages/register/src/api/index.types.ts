export interface CreateLocalUserPayload {
  nickname: string;
  avatar: string;
  password?: string | null;
}

export interface LocalUserProfile {
  uuid: string;
  nickname: string;
  avatar: string;
  hasPassword: boolean;
  currentWorkspaceUuid?: string | null;
  currentTeamUuid?: string | null;
}
