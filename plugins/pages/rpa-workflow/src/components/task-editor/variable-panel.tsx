import { useTranslation } from 'react-i18next';
import { Copy, Plus, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Switch } from '@/components/ui/switch';
import type { TaskVariable } from './task-settings-form';

export interface RuntimeVariableItem {
  name: string;
  value: string;
  source: 'extract' | 'script';
}

interface VariablePanelProps {
  globalVariables: TaskVariable[];
  runtimeVariables: RuntimeVariableItem[];
  onGlobalVariablesChange: (variables: TaskVariable[]) => void;
  embedded?: boolean;
}

function createGlobalVariable(): TaskVariable {
  return {
    id: 'var-' + Math.random().toString(36).slice(2, 8),
    name: '',
    value: '',
    promptOnRun: false,
    required: false,
  };
}

export function VariablePanel({
  globalVariables,
  runtimeVariables,
  onGlobalVariablesChange,
  embedded = false,
}: VariablePanelProps) {
  const { t } = useTranslation('rpa');
  const runtimeVariableMap = Array.from(
    runtimeVariables.reduce(
      (map, variable) => map.set(variable.name, variable),
      new Map<string, RuntimeVariableItem>()
    ).values()
  );

  const handleAddGlobalVariable = () => {
    onGlobalVariablesChange([...globalVariables, createGlobalVariable()]);
  };

  const handleUpdateGlobalVariable = (
    id: string,
    key: keyof Pick<TaskVariable, 'name' | 'value' | 'promptOnRun' | 'required'>,
    value: string | boolean
  ) => {
    onGlobalVariablesChange(
      globalVariables.map((variable) =>
        variable.id === id ? { ...variable, [key]: value } : variable
      )
    );
  };

  const handlePatchGlobalVariable = (id: string, patch: Partial<TaskVariable>) => {
    onGlobalVariablesChange(
      globalVariables.map((variable) => (variable.id === id ? { ...variable, ...patch } : variable))
    );
  };

  const handleDeleteGlobalVariable = (id: string) => {
    onGlobalVariablesChange(globalVariables.filter((variable) => variable.id !== id));
  };

  const handleCopyReference = async (name: string) => {
    try {
      await navigator.clipboard.writeText('{{' + name + '}}');
    } catch {
      // ignore clipboard failures
    }
  };

  return (
    <div
      className={
        embedded
          ? 'bg-background flex flex-1 min-h-0 flex-col overflow-hidden'
          : 'w-80 border-l border-border bg-background flex flex-col min-h-0 overflow-hidden'
      }
    >
      {!embedded ? (
        <div className="px-3 py-2 border-b border-border flex items-center justify-between shrink-0">
          <h3 className="text-xs font-semibold text-foreground">
            {t('editor.variablePanel.title', { defaultValue: '变量' })}
          </h3>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-7 px-2 text-[10px]"
            onClick={handleAddGlobalVariable}
          >
            <Plus className="h-3 w-3" />
            {t('editor.variablePanel.addVariable', { defaultValue: '新增变量' })}
          </Button>
        </div>
      ) : null}

      <ScrollArea className="flex-1 min-h-0 overflow-y-auto">
        <div className="p-3 space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label className="text-xs">
                {t('editor.variablePanel.globalTitle', { defaultValue: '全局变量' })}
              </Label>
              {embedded ? (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-[10px]"
                  onClick={handleAddGlobalVariable}
                >
                  <Plus className="h-3 w-3" />
                  {t('editor.variablePanel.addVariable', { defaultValue: '新增变量' })}
                </Button>
              ) : null}
            </div>
            <div className="space-y-2 rounded-md border border-border bg-muted/10 p-3">
              {globalVariables.length === 0 ? (
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.variablePanel.emptyGlobal', {
                    defaultValue: '暂无全局变量。你可以在这里新增常量，并在流程中通过',
                  })}{' '}
                  <span className="font-mono">{'{{变量名}}'}</span> 使用。
                </div>
              ) : (
                globalVariables.map((variable) => (
                  <div key={variable.id} className="rounded-md border border-border bg-background p-2 space-y-2">
                    <div className="flex items-center gap-2">
                      <Input
                        value={variable.name}
                        onChange={(e) => handleUpdateGlobalVariable(variable.id, 'name', e.target.value)}
                        placeholder={t('editor.variablePanel.namePlaceholder', {
                          defaultValue: '变量名',
                        })}
                        className="h-8 text-xs"
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive"
                        onClick={() => handleDeleteGlobalVariable(variable.id)}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                    <Input
                      value={variable.value}
                      onChange={(e) => handleUpdateGlobalVariable(variable.id, 'value', e.target.value)}
                      placeholder={t('editor.variablePanel.valuePlaceholder', {
                        defaultValue: '变量值',
                      })}
                      className="h-8 text-xs"
                    />
                    <div className="grid gap-2 rounded-md border border-border/60 bg-muted/20 px-2 py-2">
                      <div className="flex items-start justify-between gap-3">
                        <div className="text-[11px] font-medium text-foreground">
                          {t('editor.variablePanel.promptOnRun', {
                            defaultValue: '输入参数',
                          })}
                        </div>
                        <Switch
                          checked={variable.promptOnRun}
                          onCheckedChange={(checked) =>
                            handlePatchGlobalVariable(variable.id, {
                              promptOnRun: checked,
                              required: checked ? variable.required : false,
                            })
                          }
                          className="mt-0.5 scale-75"
                        />
                      </div>
                      <div className="flex items-start justify-between gap-3">
                        <div className="text-[11px] font-medium text-foreground">
                          {t('editor.variablePanel.required', {
                            defaultValue: '必填',
                          })}
                        </div>
                        <Switch
                          checked={variable.promptOnRun && variable.required}
                          disabled={!variable.promptOnRun}
                          onCheckedChange={(checked) =>
                            handlePatchGlobalVariable(variable.id, { required: checked })
                          }
                          className="mt-0.5 scale-75"
                        />
                      </div>
                    </div>
                    {variable.name.trim() ? (
                      <div className="flex items-center justify-between gap-2 rounded-md bg-muted/30 px-2 py-1.5">
                        <span className="truncate text-[10px] text-muted-foreground font-mono">
                          {'{{' + variable.name.trim() + '}}'}
                        </span>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-6 px-2 text-[10px]"
                          onClick={() => void handleCopyReference(variable.name.trim())}
                        >
                          <Copy className="h-3 w-3" />
                          {t('editor.variablePanel.copyReference', {
                            defaultValue: '复制引用',
                          })}
                        </Button>
                      </div>
                    ) : null}
                  </div>
                ))
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label className="text-xs">
              {t('editor.variablePanel.runtimeTitle', { defaultValue: '运行结果' })}
            </Label>
            <div className="space-y-2 rounded-md border border-border bg-muted/10 p-3">
              {runtimeVariableMap.length === 0 ? (
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.variablePanel.emptyRuntime', {
                    defaultValue:
                      '暂无运行结果。执行脚本、提取页面数据后，会在这里显示可引用的结果变量。',
                  })}
                </div>
              ) : (
                runtimeVariableMap.map((variable) => (
                  <div key={variable.name} className="rounded-md border border-border bg-background p-2 space-y-2">
                    <div className="flex items-center justify-between gap-2">
                      <div className="min-w-0">
                        <div className="truncate text-xs font-medium text-foreground">{variable.name}</div>
                        <div className="text-[10px] text-muted-foreground">
                          {variable.source === 'script' ? '执行脚本' : '提取页面数据'}
                        </div>
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="h-6 px-2 text-[10px] shrink-0"
                        onClick={() => void handleCopyReference(variable.name)}
                      >
                        <Copy className="h-3 w-3" />
                        {t('editor.variablePanel.copyReference', {
                          defaultValue: '复制引用',
                        })}
                      </Button>
                    </div>
                    <div className="rounded-md bg-muted/30 px-2 py-2 text-[10px] text-muted-foreground break-all">
                      {variable.value ||
                        t('editor.variablePanel.emptyValue', { defaultValue: '空值' })}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
