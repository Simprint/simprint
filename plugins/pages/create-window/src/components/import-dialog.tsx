import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { isSuccess, post } from '@/lib/request';

interface ImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ImportDialog({ open, onOpenChange }: ImportDialogProps) {
  const { t } = useTranslation('create-window');
  const [importJson, setImportJson] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const handleImport = async () => {
    if (!importJson.trim()) {
      toast.warning('请输入要导入的 JSON 数据');
      return;
    }

    try {
      const data = JSON.parse(importJson);
      setSubmitting(true);

      const environments = Array.isArray(data) ? data : [data];
      const response = await post<unknown[]>('environments/batch-create', { environments });

      if (!isSuccess(response)) {
        throw new Error(response.message || '导入失败');
      }

      const imported = Array.isArray(response.data) ? response.data.length : environments.length;
      toast.success(`成功导入 ${imported} 个窗口`);
      setImportJson('');
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : '导入失败，请检查 JSON 格式');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px] gap-0 p-3">
        <DialogHeader className="mb-3">
          <DialogTitle className="text-sm font-semibold mb-0">{t('import.title')}</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <p className="text-xs text-muted-foreground">{t('import.description')}</p>
          <div className="space-y-2">
            <Label htmlFor="import-json">JSON 数据</Label>
            <textarea
              id="import-json"
              value={importJson}
              onChange={(e) => setImportJson(e.target.value)}
              className="w-full h-96 px-3 py-2 text-xs font-mono bg-background border border-border rounded resize-none"
              placeholder={t('import.placeholder')}
            />
          </div>
        </div>
        <DialogFooter className="mt-4 gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setImportJson('');
              onOpenChange(false);
            }}
            disabled={submitting}
          >
            {t('actions.cancel')}
          </Button>
          <Button onClick={handleImport} disabled={submitting} size="sm">
            {submitting ? t('actions.submitting') : t('import.title')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
