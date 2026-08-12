import { useEffect, useMemo, useState } from 'react';
import { Download, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';

import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import {
  APP_UPDATE_AVAILABLE_EVENT,
  APP_UPDATE_DIALOG_EVENT,
  checkForAppUpdate,
  getAvailableAppUpdate,
  installAvailableAppUpdate,
} from '@/lib/app-updater';

export function AppUpdateButton() {
  const { t } = useTranslation('appLayout');
  const [update, setUpdate] = useState<Update | null>(() => getAvailableAppUpdate());
  const [open, setOpen] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [downloaded, setDownloaded] = useState(0);
  const [total, setTotal] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const handleAvailable = (event: Event) => {
      const availableUpdate = (event as CustomEvent<Update>).detail;
      setUpdate(availableUpdate);
      setError(null);
      setOpen(true);
    };
    const handleOpen = () => {
      if (getAvailableAppUpdate()) {
        setOpen(true);
      }
    };

    window.addEventListener(APP_UPDATE_AVAILABLE_EVENT, handleAvailable);
    window.addEventListener(APP_UPDATE_DIALOG_EVENT, handleOpen);

    // One non-blocking check is performed after the authenticated main layout mounts.
    void checkForAppUpdate().catch((checkError) => {
      console.warn('[Updater] Automatic update check failed:', checkError);
    });

    return () => {
      window.removeEventListener(APP_UPDATE_AVAILABLE_EVENT, handleAvailable);
      window.removeEventListener(APP_UPDATE_DIALOG_EVENT, handleOpen);
    };
  }, []);

  const progress = useMemo(() => {
    if (!total || total <= 0) return null;
    return Math.min(100, Math.round((downloaded / total) * 100));
  }, [downloaded, total]);

  const handleDownloadEvent = (event: DownloadEvent) => {
    if (event.event === 'Started') {
      setDownloaded(0);
      setTotal(event.data.contentLength ?? null);
    } else if (event.event === 'Progress') {
      setDownloaded((value) => value + event.data.chunkLength);
    }
  };

  const handleInstall = async () => {
    setInstalling(true);
    setDownloaded(0);
    setTotal(null);
    setError(null);

    try {
      await installAvailableAppUpdate(handleDownloadEvent);
    } catch (installError) {
      console.error('[Updater] Update installation failed:', installError);
      setError(t('update.installFailed'));
      setInstalling(false);
    }
  };

  if (!update) {
    return null;
  }

  return (
    <>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            className="relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-sm text-primary/90 outline-none transition-all duration-200 ease-in-out hover:bg-accent/80 hover:text-primary"
            title={t('update.available')}
            onClick={() => setOpen(true)}
          >
            <RefreshCw className="h-4 w-4" />
            <span className="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full bg-primary" />
          </button>
        </TooltipTrigger>
        <TooltipContent>{t('update.available')}</TooltipContent>
      </Tooltip>

      <AlertDialog open={open} onOpenChange={(nextOpen) => !installing && setOpen(nextOpen)}>
        <AlertDialogContent className="flex max-h-[min(82vh,720px)] w-[min(92vw,680px)] max-w-none flex-col gap-0 overflow-hidden p-0">
          <AlertDialogHeader className="shrink-0 border-b px-6 py-5">
            <div className="flex items-start gap-3">
              <div className="mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
                <Download className="h-5 w-5" />
              </div>
              <div className="min-w-0">
                <AlertDialogTitle>{t('update.dialogTitle')}</AlertDialogTitle>
                <AlertDialogDescription className="mt-1">
                  {t('update.versionChange', {
                    current: update.currentVersion,
                    next: update.version,
                  })}
                </AlertDialogDescription>
              </div>
            </div>
          </AlertDialogHeader>

          <div className="min-h-0 overflow-y-auto px-6 py-5">
            <p className="mb-2 text-sm font-medium text-foreground">{t('update.releaseNotes')}</p>
            <div className="max-h-[42vh] overflow-y-auto rounded-lg border bg-muted/30 p-4 text-sm leading-6 whitespace-pre-wrap break-words text-muted-foreground">
              {update.body?.trim() || t('update.noReleaseNotes')}
            </div>

            {installing ? (
              <div className="mt-5 space-y-2">
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>{t('update.downloading')}</span>
                  <span>{progress == null ? t('update.preparing') : `${progress}%`}</span>
                </div>
                <Progress value={progress ?? 0} />
              </div>
            ) : null}

            {error ? <p className="mt-4 text-sm text-destructive">{error}</p> : null}
          </div>

          <AlertDialogFooter className="shrink-0 border-t px-6 py-4">
            <AlertDialogCancel disabled={installing}>{t('update.later')}</AlertDialogCancel>
            <Button onClick={() => void handleInstall()} disabled={installing}>
              {installing ? t('update.installing') : t('update.downloadAndInstall')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
