import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export const APP_UPDATE_AVAILABLE_EVENT = 'simprint:app-update-available';
export const APP_UPDATE_DIALOG_EVENT = 'simprint:open-app-update-dialog';

let cachedUpdate: Update | null | undefined;
let pendingCheck: Promise<Update | null> | null = null;

function publishAvailableUpdate(update: Update): void {
  window.dispatchEvent(
    new CustomEvent(APP_UPDATE_AVAILABLE_EVENT, {
      detail: update,
    })
  );
}

export function getAvailableAppUpdate(): Update | null {
  return cachedUpdate ?? null;
}

export async function checkForAppUpdate(options?: { force?: boolean }): Promise<Update | null> {
  if (cachedUpdate) {
    return cachedUpdate;
  }
  if (cachedUpdate === null && !options?.force) {
    return null;
  }
  if (pendingCheck) {
    return pendingCheck;
  }

  pendingCheck = check({ timeout: 30_000 })
    .then((update) => {
      cachedUpdate = update;
      if (update) {
        publishAvailableUpdate(update);
      }
      return update;
    })
    .finally(() => {
      pendingCheck = null;
    });

  return pendingCheck;
}

export function openAppUpdateDialog(): void {
  window.dispatchEvent(new Event(APP_UPDATE_DIALOG_EVENT));
}

export async function installAvailableAppUpdate(
  onEvent: (event: DownloadEvent) => void
): Promise<void> {
  const update = cachedUpdate;
  if (!update) {
    throw new Error('No application update is available');
  }

  await update.downloadAndInstall(onEvent);
  await relaunch();
}
