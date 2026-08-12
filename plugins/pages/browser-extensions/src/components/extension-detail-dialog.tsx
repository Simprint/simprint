import { useTranslation } from 'react-i18next';
import { Info } from 'lucide-react';
import { FormattedDialog, FormattedDialogFooter } from '@/components/formatted-dialog';
import { Button } from '@/components/ui/button';
import type { ExtensionItem } from '../types';
import { ExtensionIcon } from './extension-icon';

interface ExtensionDetailDialogProps {
  open: boolean;
  extension: ExtensionItem | null;
  onOpenChange: (open: boolean) => void;
}

export function ExtensionDetailDialog({
  open,
  extension,
  onOpenChange,
}: ExtensionDetailDialogProps) {
  const { t } = useTranslation('extensions');
  if (!extension) return null;

  return (
    <FormattedDialog
      open={open}
      onOpenChange={onOpenChange}
      header={{
        icon: Info,
        title: t('dialog.detail.title'),
        description: extension.name,
      }}
      contentPadding="p-5"
    >
      <div className="flex items-start gap-4 rounded-lg border bg-muted/30 p-4">
        <ExtensionIcon
          icon={extension.icon}
          source="local"
          containerClassName="h-14 w-14 rounded-lg border bg-background"
          imageClassName="rounded-lg"
          fallbackClassName="h-7 w-7"
        />
        <div className="min-w-0 flex-1 space-y-1">
          <div className="font-medium text-foreground">{extension.name}</div>
          <div className="text-xs text-muted-foreground">v{extension.version}</div>
          <p className="pt-2 text-xs leading-relaxed text-muted-foreground">
            {extension.description || t('local.noDescription')}
          </p>
        </div>
      </div>
      <FormattedDialogFooter>
        <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
          {t('dialog.detail.close')}
        </Button>
      </FormattedDialogFooter>
    </FormattedDialog>
  );
}
