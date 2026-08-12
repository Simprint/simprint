import { useCallback, useEffect, useMemo, useState } from 'react';
import { extensionRegistry } from '@slotkitjs/core';
import { Loader2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { extensionsResources } from './i18n/resources';
import { LocalExtensionLibrary } from './components/local-extension-library';
import { LocalExtensionImportDialog } from './components/local-extension-import-dialog';
import { ExtensionUninstallDialog } from './components/extension-uninstall-dialog';
import { ExtensionToggleDialog } from './components/extension-toggle-dialog';
import { ExtensionDetailDialog } from './components/extension-detail-dialog';
import {
  disableLocalExtension,
  enableLocalExtension,
  importLocalExtensionCrx,
  importLocalExtensionStoreUrl,
  installLocalExtension,
  listLocalExtensions,
  removeLocalExtension,
  type Extension,
} from './api';
import type { ExtensionItem } from './types';

function toExtensionItem(extension: Extension): ExtensionItem {
  return {
    id: extension.id,
    uuid: extension.id,
    recordId: extension.recordId,
    extensionId: extension.extensionId,
    name: extension.name,
    description: extension.description,
    version: extension.version,
    icon: extension.icon || '',
    browser: extension.browser as ExtensionItem['browser'],
    status: extension.status,
    source: 'local',
    author: extension.author,
    homepage: extension.homepage,
    downloads: extension.downloads,
    rating: extension.rating,
    updatedAt: extension.updatedAt,
    createdAt: extension.createdAt,
    fileSize: extension.fileSize,
    permissions: extension.permissions,
    hash: extension.hash,
    scope: extension.scope,
    category: extension.category,
  };
}

const BrowserExtensionsPage: React.FC = () => {
  const { t } = useTranslation('extensions');
  const [extensions, setExtensions] = useState<ExtensionItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<ExtensionItem | null>(null);
  const [toggleTarget, setToggleTarget] = useState<ExtensionItem | null>(null);
  const [toggleAction, setToggleAction] = useState<'disable' | 'enable'>('disable');
  const [detailTarget, setDetailTarget] = useState<ExtensionItem | null>(null);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      setExtensions((await listLocalExtensions()).map(toExtensionItem));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const byId = useMemo(
    () => new Map(extensions.map((extension) => [extension.id, extension])),
    [extensions]
  );

  const recordId = (extension: ExtensionItem) => extension.recordId || extension.id;

  const handleImport = async ({
    mode,
    crxPath,
    storeUrl,
  }: {
    mode: 'file' | 'storeUrl';
    crxPath?: string;
    storeUrl?: string;
  }) => {
    setImporting(true);
    try {
      const result =
        mode === 'file'
          ? await importLocalExtensionCrx(crxPath || '')
          : await importLocalExtensionStoreUrl(storeUrl || '');
      setImportOpen(false);
      if (result.importState === 'alreadyInstalled') {
        toast.info(t('dialog.localImport.alreadyInstalled'));
      } else if (result.importState === 'alreadyExists') {
        toast.info(t('dialog.localImport.alreadyExists'));
      } else {
        toast.success(t('dialog.localImport.success'));
      }
      await refresh();
    } finally {
      setImporting(false);
    }
  };

  const handleInstall = async (extension: ExtensionItem) => {
    await installLocalExtension(recordId(extension));
    toast.success(t('local.installSuccess'));
    await refresh();
  };

  const confirmRemove = async () => {
    if (!removeTarget) return;
    await removeLocalExtension(recordId(removeTarget));
    setRemoveTarget(null);
    await refresh();
  };

  const confirmToggle = async () => {
    if (!toggleTarget) return;
    const command = toggleAction === 'disable' ? disableLocalExtension : enableLocalExtension;
    await command(recordId(toggleTarget));
    setToggleTarget(null);
    await refresh();
  };

  return (
    <div className="relative flex h-[calc(100vh-50px)] flex-col">
      {loading ? (
        <div className="flex flex-1 items-center justify-center text-muted-foreground">
          <Loader2 className="h-5 w-5 animate-spin" />
        </div>
      ) : (
        <>
          {error && (
            <div className="border-b bg-destructive/10 px-4 py-2 text-xs text-destructive">
              {t('error', { message: error })}
            </div>
          )}
          <LocalExtensionLibrary
            extensions={extensions}
            importing={importing}
            onImport={() => setImportOpen(true)}
            onInstall={(extension) => void handleInstall(byId.get(extension.id) || extension)}
            onDisable={(extension) => {
              setToggleTarget(extension);
              setToggleAction('disable');
            }}
            onEnable={(extension) => {
              setToggleTarget(extension);
              setToggleAction('enable');
            }}
            onRemove={setRemoveTarget}
            onViewDetails={setDetailTarget}
          />
        </>
      )}

      <LocalExtensionImportDialog
        open={importOpen}
        importing={importing}
        onOpenChange={setImportOpen}
        onSubmit={handleImport}
      />
      <ExtensionUninstallDialog
        open={!!removeTarget}
        extension={removeTarget}
        action="remove"
        onOpenChange={(open) => !open && setRemoveTarget(null)}
        onConfirm={confirmRemove}
      />
      <ExtensionToggleDialog
        open={!!toggleTarget}
        extension={toggleTarget}
        action={toggleAction}
        onOpenChange={(open) => !open && setToggleTarget(null)}
        onConfirm={confirmToggle}
      />
      <ExtensionDetailDialog
        open={!!detailTarget}
        extension={detailTarget}
        onOpenChange={(open) => !open && setDetailTarget(null)}
      />
    </div>
  );
};

try {
  extensionRegistry.contribute('routes', {
    contributorId: 'browser-extensions',
    value: { path: '/extensions', Component: BrowserExtensionsPage },
    priority: 10,
  });
} catch (error) {
  console.warn('[browser-extensions] Failed to contribute route:', error);
}

try {
  extensionRegistry.contribute('i18n:resources', {
    contributorId: 'browser-extensions',
    value: { namespace: 'extensions', resources: extensionsResources },
    priority: 10,
  });
} catch (error) {
  console.warn('[browser-extensions] Failed to contribute i18n resources:', error);
}

export default {
  id: 'browser-extensions',
  name: 'Browser Extensions',
  version: '1.0.0',
  component: BrowserExtensionsPage,
  slots: [],
};
