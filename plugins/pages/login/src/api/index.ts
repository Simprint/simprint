import { invoke } from '@/lib/tauri';
import type { LocalUserProfile } from './index.types';

export * from './index.types';

export async function listLocalUsers(): Promise<LocalUserProfile[]> {
  return invoke<LocalUserProfile[]>('list_local_users');
}

export async function loginLocalUser(
  userUuid: string,
  password?: string
): Promise<LocalUserProfile> {
  return invoke<LocalUserProfile>('login_local_user', {
    payload: { userUuid, password: password || null },
  });
}
