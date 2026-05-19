import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Play, RotateCcw, X } from 'lucide-react';
import { FormattedDialog, FormattedDialogFooter } from '@/components/formatted-dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import type { RpaTask } from '../types';
import { extractTaskInputVariables, type RpaTaskDetailDto } from '../api';

interface RpaRunDialogProps {
  open: boolean;
  loading?: boolean;
  pending: { task: RpaTask; detail: RpaTaskDetailDto } | null;
  onOpenChange: (open: boolean) => void;
  onConfirm: (variableOverrides?: Record<string, string>) => Promise<void> | void;
}

export function RpaRunDialog({
  open,
  loading = false,
  pending,
  onOpenChange,
  onConfirm,
}: RpaRunDialogProps) {
  const { t } = useTranslation('rpa');
  const environmentCount = pending?.detail.environment_uuids.length ?? 0;
  const strategy = pending?.detail.task;
  const canRun = Boolean(pending && environmentCount > 0);
  const inputVariables = useMemo(
    () => (pending ? extractTaskInputVariables(pending.detail) : []),
    [pending]
  );
  const [inputValues, setInputValues] = useState<Record<string, string>>({});
  const [inputErrors, setInputErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    setInputValues(
      Object.fromEntries(inputVariables.map((variable) => [variable.name, variable.value]))
    );
    setInputErrors({});
  }, [inputVariables, open, pending]);

  const handleConfirm = () => {
    const nextErrors = Object.fromEntries(
      inputVariables.flatMap((variable) => {
        if (!variable.required) {
          return [];
        }

        const value = inputValues[variable.name] ?? '';
        return value.trim()
          ? []
          : [
              [
                variable.name,
                t('editor.variablePanel.runDialog.requiredError', {
                  defaultValue: '请输入 {{name}}',
                  name: variable.name,
                }),
              ],
            ];
      })
    );

    if (Object.keys(nextErrors).length > 0) {
      setInputErrors(nextErrors);
      return;
    }

    void onConfirm(inputValues);
  };

  return (
    <FormattedDialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!loading) {
          onOpenChange(isOpen);
        }
      }}
      minWidth="min-w-[560px]"
      header={{
        icon: Play,
        iconColor: 'text-primary',
        title: t('dialog.run.title', { defaultValue: '开始执行任务' }),
        description: pending
          ? t('dialog.run.descriptionWithEnvCount', {
              defaultValue: '将在 {{count}} 个绑定环境中正式执行该任务。',
              count: environmentCount,
            })
          : t('dialog.run.description', {
              defaultValue: '将在已绑定环境上正式执行该任务。',
            }),
        gradient: 'bg-gradient-to-r from-primary/10 via-blue-500/10 to-primary/10',
        className: 'border-b border-primary/20',
      }}
      contentPadding="p-5"
      contentClassName="max-h-[min(70vh,720px)] overflow-y-auto pr-3"
    >
      <div className="space-y-4 text-sm">
        <div className="space-y-1">
          <div className="text-base font-semibold text-foreground">
            {pending?.task.name || t('dialog.run.noTask', { defaultValue: '未选择任务' })}
          </div>
          {pending?.task.description ? (
            <div className="text-xs text-muted-foreground">{pending.task.description}</div>
          ) : null}
        </div>

        {inputVariables.length > 0 ? (
          <div className="rounded-md border border-border/50 bg-muted/20 p-3">
            <div className="mb-3 text-xs font-medium text-foreground">
              {t('editor.variablePanel.runDialog.title', { defaultValue: '输入参数' })}
            </div>
            <div className="space-y-3">
              {inputVariables.map((variable) => {
                const error = inputErrors[variable.name];

                return (
                  <div key={variable.name} className="space-y-1.5">
                    <div className="flex items-center gap-2">
                      <label className="text-xs font-medium text-foreground">
                        {variable.name}
                        {variable.required ? <span className="ml-1 text-destructive">*</span> : null}
                      </label>
                      <span className="text-[10px] font-mono text-muted-foreground">
                        {'{{' + variable.name + '}}'}
                      </span>
                    </div>
                    <Input
                      value={inputValues[variable.name] ?? ''}
                      onChange={(event) => {
                        const value = event.target.value;
                        setInputValues((prev) => ({ ...prev, [variable.name]: value }));
                        setInputErrors((prev) => {
                          if (!prev[variable.name]) {
                            return prev;
                          }

                          const next = { ...prev };
                          delete next[variable.name];
                          return next;
                        });
                      }}
                      placeholder={t('editor.variablePanel.runDialog.valuePlaceholder', {
                        defaultValue: '请输入参数值',
                      })}
                      className={error ? 'border-destructive focus-visible:ring-destructive' : ''}
                    />
                    {error ? (
                      <div className="text-[11px] text-destructive">{error}</div>
                    ) : null}
                  </div>
                );
              })}
            </div>
          </div>
        ) : null}

        <div className="rounded-md border border-border/50 bg-muted/20 p-3">
          <div className="mb-2 text-xs font-medium text-foreground">
            {t('dialog.run.strategy', { defaultValue: '执行策略' })}
          </div>
          <div className="flex flex-wrap gap-1.5">
            <Badge variant="outline" className="gap-1">
              <RotateCcw className="h-3 w-3" />
              {t('dialog.run.retryCount', {
                defaultValue: '重试 {{count}} 次',
                count: strategy?.retry_count ?? 3,
              })}
            </Badge>
            <Badge variant="outline">
              {t('dialog.run.retryInterval', {
                defaultValue: '间隔 {{seconds}} 秒',
                seconds: strategy?.retry_interval ?? 5,
              })}
            </Badge>
            <Badge variant="outline">
              {t('dialog.run.timeout', {
                defaultValue: '超时 {{seconds}} 秒',
                seconds: strategy?.timeout ?? 300,
              })}
            </Badge>
            {strategy?.stop_on_error ? (
              <Badge variant="outline">
                {t('dialog.run.stopOnError', { defaultValue: '出错时停止' })}
              </Badge>
            ) : null}
          </div>
        </div>

        {pending && !canRun ? (
          <div className="rounded-md border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
            {t('dialog.run.environmentRequired', {
              defaultValue: '请先为任务绑定环境后再执行。',
            })}
          </div>
        ) : null}
      </div>

      <FormattedDialogFooter>
        <Button type="button" variant="outline" size="sm" onClick={() => onOpenChange(false)} disabled={loading}>
          <X className="mr-1.5 h-4 w-4" />
          {t('dialog.run.cancel', { defaultValue: '取消' })}
        </Button>
        <Button type="button" size="sm" onClick={handleConfirm} disabled={loading || !canRun}>
          <Play className="mr-1.5 h-4 w-4" />
          {t('dialog.run.confirm', { defaultValue: '开始执行' })}
        </Button>
      </FormattedDialogFooter>
    </FormattedDialog>
  );
}
