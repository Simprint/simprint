import { invoke } from '@/lib/tauri';
import type { CreateLocalUserPayload, LocalUserProfile } from './index.types';

export * from './index.types';

export async function createLocalUser(
  payload: CreateLocalUserPayload
): Promise<LocalUserProfile> {
  return invoke<LocalUserProfile>('create_local_user', { payload });
}
