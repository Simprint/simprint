import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import {
  HardDrive,
  FolderOpen,
  Download,
  RefreshCcw,
  Database,
  FileBox,
  FileText,
  Copy,
  ExternalLink,
  Settings,
  RotateCcw,
  Loader2,
  X,
} from 'lucide-react';
import { getVersion } from '@tauri-apps/api/app';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { checkForAppUpdate, openAppUpdateDialog } from '@/lib/app-updater';
import { FormattedDialog, FormattedDialogFooter } from '@/components/formatted-dialog';
import { SettingCard } from './setting-card';
import { StoragePanelSkeleton } from './storage-panel-skeleton';
import { getStorageSettings, setStorageSettings } from '../../../../services/store/src';
import { getStorageDefaultPaths, getDirectorySizes } from '../api/storage';
import { openPath } from '@tauri-apps/plugin-opener';
import { open as openFolderDialog } from '@tauri-apps/plugin-dialog';
import type { DirectorySizeCache, StoragePathKey } from '../../../../services/store/src';

interface StorageItemDef {
  id: string;
  key: StoragePathKey | 'profilesPath' | 'downloadsPath';
  nameKey: string;
  descKey: string;
  icon: React.ElementType;
  color: string;
  readonly?: boolean; // 只读目录，不允许用户配置
}

const STORAGE_ITEM_DEFS: StorageItemDef[] = [
  {
    id: 'profiles',
    key: 'profilesPath',
    nameKey: 'storageProfiles',
    descKey: 'storageProfilesDesc',
    icon: Database,
    color: 'bg-blue-500',
    readonly: true,
  },
  {
    id: 'cache',
    key: 'cachePath',
    nameKey: 'storageCache',
    descKey: 'storageCacheDesc',
    icon: HardDrive,
    color: 'bg-amber-500',
  },
  {
    id: 'logs',
    key: 'logsPath',
    nameKey: 'storageLogs',
    descKey: 'storageLogsDesc',
    icon: FileText,
    color: 'bg-emerald-500',
  },
  {
    id: 'downloads',
    key: 'downloadsPath',
    nameKey: 'storageDownloads',
    descKey: 'storageDownloadsDesc',
    icon: FileBox,
    color: 'bg-purple-500',
    readonly: true,
  },
];

const EMPTY_SIZES = STORAGE_ITEM_DEFS.map(() => 0);

function isDirectorySizeCacheValid(
  cache: DirectorySizeCache | undefined,
  paths: string[]
): cache is DirectorySizeCache {
  return Boolean(
    cache &&
    cache.paths.length === paths.length &&
    cache.sizes.length === paths.length &&
    cache.paths.every((path, index) => path === paths[index])
  );
}

/**
 * 存储与更新面板
 */
export const StoragePanel: React.FC = () => {
  const { t } = useTranslation('settings');
  const [appVersion, setAppVersion] = useState<string | null>(null);

  const [defaultPaths, setDefaultPaths] = useState<{
    app_base: string;
    profiles: string;
    cache: string;
    logs: string;
    downloads: string;
  } | null>(null);
  const [paths, setPaths] = useState<string[]>([]);
  const [sizes, setSizes] = useState<number[]>(EMPTY_SIZES);
  const [pathsLoading, setPathsLoading] = useState(true);
  const [sizesLoading, setSizesLoading] = useState(true);
  const [sizesRefreshing, setSizesRefreshing] = useState(false);
  const [sizesUpdatedAt, setSizesUpdatedAt] = useState<number | null>(null);
  const [restoreConfirmOpen, setRestoreConfirmOpen] = useState(false);
  const [restoreSubmitting, setRestoreSubmitting] = useState(false);

  const [hasUpdate, setHasUpdate] = useState(false);
  const [updateChecking, setUpdateChecking] = useState(false);
  const [updateChecked, setUpdateChecked] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void getVersion()
      .then((v) => {
        if (!cancelled) setAppVersion(v);
      })
      .catch(() => {
        if (!cancelled) setAppVersion(null);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const loadPathsAndSizes = useCallback(async () => {
    setPathsLoading(true);
    try {
      const [defs, storage] = await Promise.all([getStorageDefaultPaths(), getStorageSettings()]);
      setDefaultPaths(defs);

      const resolvedPaths = STORAGE_ITEM_DEFS.map((d) => {
        const custom = storage[d.key as keyof typeof storage];
        if (typeof custom === 'string' && custom.trim()) return custom;
        return defs[d.id as keyof typeof defs] as string;
      });
      setPaths(resolvedPaths);
      setPathsLoading(false);

      if (isDirectorySizeCacheValid(storage.directorySizeCache, resolvedPaths)) {
        setSizes(storage.directorySizeCache.sizes);
        setSizesUpdatedAt(storage.directorySizeCache.updatedAt);
        setSizesLoading(false);
      } else {
        setSizes(EMPTY_SIZES);
        setSizesUpdatedAt(null);
        setSizesLoading(true);
      }

      setSizesRefreshing(true);
      const { sizes: freshSizes } = await getDirectorySizes(resolvedPaths);
      const updatedAt = Date.now();

      setSizes(freshSizes);
      setSizesUpdatedAt(updatedAt);
      setSizesLoading(false);

      await setStorageSettings({
        directorySizeCache: {
          paths: resolvedPaths,
          sizes: freshSizes,
          updatedAt,
        },
      });
    } catch (e) {
      console.warn('[StoragePanel] load paths/sizes failed:', e);
      setPaths([]);
      setSizes(EMPTY_SIZES);
      setSizesUpdatedAt(null);
      setSizesLoading(false);
    } finally {
      setPathsLoading(false);
      setSizesRefreshing(false);
    }
  }, []);

  useEffect(() => {
    void loadPathsAndSizes();
  }, [loadPathsAndSizes]);

  const totalSizeBytes = sizes.reduce((a, b) => a + b, 0);

  const handleCopyPath = (path: string) => {
    navigator.clipboard.writeText(path);
  };

  const handleOpenFolder = useCallback((path: string) => {
    void openPath(path);
  }, []);

  const handleOpenAppFolder = useCallback(() => {
    if (defaultPaths?.app_base) {
      void openPath(defaultPaths.app_base);
    }
  }, [defaultPaths?.app_base]);

  const handleSetPath = useCallback(
    async (def: StorageItemDef) => {
      const selected = await openFolderDialog({
        directory: true,
        multiple: false,
      });
      if (selected && typeof selected === 'string') {
        await setStorageSettings({ [def.key]: selected });
        await loadPathsAndSizes();
      }
    },
    [loadPathsAndSizes]
  );

  const handleOpenRestoreConfirm = useCallback(() => {
    setRestoreConfirmOpen(true);
  }, []);

  const handleCheckUpdate = useCallback(async () => {
    setUpdateChecking(true);
    try {
      const update = await checkForAppUpdate({ force: true });
      setHasUpdate(Boolean(update));
      setUpdateChecked(true);
      if (update) {
        toast.success(t('updateAvailable'));
        openAppUpdateDialog();
      } else {
        toast.success(t('alreadyLatest'));
      }
    } catch {
      toast.error(t('checkUpdateFailed') || '检查更新失败');
      setHasUpdate(false);
      setUpdateChecked(true);
    } finally {
      setUpdateChecking(false);
    }
  }, [t]);

  const handleUpdateNow = useCallback(() => {
    openAppUpdateDialog();
  }, []);

  const handleConfirmRestore = useCallback(async () => {
    setRestoreSubmitting(true);
    setRestoreConfirmOpen(false);
    try {
      await setStorageSettings({
        cachePath: undefined,
        logsPath: undefined,
      });
      await loadPathsAndSizes();
    } finally {
      setRestoreSubmitting(false);
    }
  }, [loadPathsAndSizes]);

  const formatSize = (bytes: number) => {
    const mb = bytes / (1024 * 1024);
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${Math.round(mb)} MB`;
  };

  const formatUpdatedAt = (timestamp: number) =>
    new Date(timestamp).toLocaleString(undefined, {
      hour12: false,
    });

  return (
    <div className="space-y-6">
      {/* 存储概览 */}
      <SettingCard title={t('storageManagement')} icon={HardDrive}>
        {pathsLoading ? (
          <StoragePanelSkeleton rows={STORAGE_ITEM_DEFS.length} />
        ) : (
          <>
            {/* 总体使用情况 */}
            <div className="p-4 bg-accent/30 rounded-lg mx-2 mb-3">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <p className="text-sm font-medium text-foreground">{t('totalStorage')}</p>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    {t('used')} {formatSize(totalSizeBytes)}
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    {sizesRefreshing
                      ? t('storageRefreshingSizes')
                      : sizesUpdatedAt != null
                        ? t('storageUpdatedAt', { time: formatUpdatedAt(sizesUpdatedAt) })
                        : t('loading')}
                  </p>
                </div>
                <span className="text-lg font-semibold text-foreground">
                  {formatSize(totalSizeBytes)}
                </span>
              </div>
              {!sizesLoading && totalSizeBytes > 0 && (
                <div className="h-3 bg-muted rounded-full overflow-hidden flex">
                  {STORAGE_ITEM_DEFS.map((def, i) => {
                    const pct = totalSizeBytes > 0 ? (sizes[i] ?? 0) / totalSizeBytes : 0;
                    return (
                      <div
                        key={def.id}
                        className={`h-full ${def.color} first:rounded-l-full last:rounded-r-full`}
                        style={{ width: `${pct * 100}%` }}
                        title={`${t(def.nameKey)}: ${formatSize(sizes[i] ?? 0)}`}
                      />
                    );
                  })}
                </div>
              )}
              {!sizesLoading && (
                <div className="flex flex-wrap gap-x-4 gap-y-1 mt-3">
                  {STORAGE_ITEM_DEFS.map((def, i) => (
                    <div key={def.id} className="flex items-center gap-1.5">
                      <div className={`w-2.5 h-2.5 rounded-full ${def.color}`} />
                      <span className="text-xs text-muted-foreground">
                        {t(def.nameKey)} ({formatSize(sizes[i] ?? 0)})
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* 存储详情列表 */}
            <div className="space-y-2 px-2 pb-2">
              {STORAGE_ITEM_DEFS.map((def, i) => {
                const Icon = def.icon;
                const path = paths[i] ?? '';
                const sizeBytes = sizes[i] ?? 0;
                return (
                  <div
                    key={def.id}
                    className="flex items-center justify-between p-3 bg-accent/20 rounded-lg group"
                  >
                    <div className="flex items-center gap-3 min-w-0 flex-1">
                      <div className={`p-2 rounded-lg ${def.color}/10`}>
                        <Icon className={`w-4 h-4 ${def.color.replace('bg-', 'text-')}`} />
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <p className="text-sm font-medium text-foreground">{t(def.nameKey)}</p>
                          <span className="text-xs text-muted-foreground">
                            {sizesLoading ? t('loading') : formatSize(sizeBytes)}
                          </span>
                        </div>
                        <p className="text-xs text-muted-foreground mt-0.5 truncate" title={path}>
                          {path}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                      <button
                        onClick={() => handleCopyPath(path)}
                        className="p-1.5 rounded hover:bg-accent/50 transition-colors opacity-0 group-hover:opacity-100"
                        title={t('copyPath')}
                      >
                        <Copy className="w-3.5 h-3.5 text-muted-foreground" />
                      </button>
                      <button
                        onClick={() => handleOpenFolder(path)}
                        className="p-1.5 rounded hover:bg-accent/50 transition-colors"
                        title={t('openFolder')}
                      >
                        <ExternalLink className="w-3.5 h-3.5 text-muted-foreground" />
                      </button>
                      {!def.readonly && (
                        <button
                          onClick={() => handleSetPath(def)}
                          className="p-1.5 rounded hover:bg-accent/50 transition-colors"
                          title={t('storageSetPath')}
                        >
                          <Settings className="w-3.5 h-3.5 text-muted-foreground" />
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </>
        )}

        {/* 快捷操作 */}
        <div className="flex items-center gap-3 px-2 pt-2 mx-2">
          <button
            onClick={handleOpenAppFolder}
            className="flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 rounded-lg transition-colors"
          >
            <FolderOpen className="w-4 h-4" />
            {t('openAppFolder')}
          </button>
          <button
            onClick={handleOpenRestoreConfirm}
            disabled={pathsLoading}
            className="flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 rounded-lg transition-colors"
          >
            <RotateCcw className="w-4 h-4" />
            {t('storageRestoreDefaults')}
          </button>
        </div>
      </SettingCard>

      {/* 恢复默认确认弹窗 */}
      <FormattedDialog
        open={restoreConfirmOpen}
        onOpenChange={setRestoreConfirmOpen}
        minWidth="min-w-[440px]"
        header={{
          icon: RotateCcw,
          title: t('storageRestoreDefaults'),
          description: t('storageRestoreDefaultsConfirm'),
        }}
        contentPadding="p-5"
      >
        <FormattedDialogFooter>
          <Button
            variant="outline"
            size="sm"
            className="text-xs"
            onClick={() => setRestoreConfirmOpen(false)}
            disabled={restoreSubmitting}
          >
            <X className="h-4 w-4 mr-1.5" />
            {t('cancel')}
          </Button>
          <Button
            size="sm"
            className="text-xs bg-primary text-primary-foreground hover:bg-primary/90 border-0"
            onClick={() => void handleConfirmRestore()}
            disabled={restoreSubmitting}
          >
            {restoreSubmitting ? (
              <>
                <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
                {t('storageRestoring')}
              </>
            ) : (
              <>
                <RotateCcw className="h-4 w-4 mr-1.5" />
                {t('storageRestoreDefaults')}
              </>
            )}
          </Button>
        </FormattedDialogFooter>
      </FormattedDialog>

      {/* 版本更新 */}
      <SettingCard title={t('versionUpdate')} icon={Download}>
        {/* 当前版本 */}
        <div className="flex items-center justify-between p-4 bg-accent/30 rounded-lg mx-2 my-2">
          <div>
            <p className="text-sm font-medium text-foreground">{t('currentVersion')}</p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {appVersion != null ? `v${appVersion}` : '—'}
            </p>
          </div>
          {hasUpdate ? (
            <button
              onClick={handleUpdateNow}
              className="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-primary rounded hover:bg-primary/10 transition-colors"
            >
              <RefreshCcw className="w-3 h-3" />
              {t('updateNow')}
            </button>
          ) : (
            <button
              onClick={handleCheckUpdate}
              disabled={updateChecking}
              className="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-primary rounded hover:bg-primary/10 transition-colors disabled:opacity-50"
            >
              {updateChecking ? (
                <>
                  <Loader2 className="w-3 h-3 animate-spin" />
                  {t('checking')}
                </>
              ) : (
                <>
                  <RefreshCcw className="w-3 h-3" />
                  {updateChecked ? t('alreadyLatest') : t('checkUpdate')}
                </>
              )}
            </button>
          )}
        </div>
      </SettingCard>
    </div>
  );
};
