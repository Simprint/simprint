import { useEffect, useState } from 'react';
import { BadgeCheck, Clock3, Hand, Plus, RefreshCw, Workflow, X } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion';
import { NumberStepInput } from '../number-step-input';
import { listRpaEnvironments } from '../../api';
import { EnvironmentSelectorDialog, type Environment } from './environment-selector-dialog';

export interface TaskVariable {
  id: string;
  name: string;
  value: string;
  promptOnRun: boolean;
  required: boolean;
}

export interface TaskConfig {
  name: string;
  description: string;
  tags: string[];
  triggerType: 'manual' | 'scheduled' | 'event';
  schedule?: string;
  cronExpression?: string;
  selectedEnvironments: string[];
  runMode: 'sequential' | 'parallel';
  globalVariables: TaskVariable[];
  retryCount: number;
  retryInterval: number;
  timeout: number;
  concurrency: number;
  stopOnError: boolean;
  closeBrowserOnComplete: boolean;
  notifyOnComplete: boolean;
  notifyOnError: boolean;
}

interface TaskSettingsFormProps {
  config: TaskConfig;
  onConfigChange: (config: TaskConfig) => void;
}

function SummaryBadge({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
}) {
  return (
    <div className="inline-flex items-center gap-2 rounded-full border border-border/70 bg-background/90 px-3 py-1.5">
      <Icon className="h-3.5 w-3.5 text-muted-foreground" />
      <span className="text-[11px] text-muted-foreground">{label}</span>
      <span className="text-[11px] font-medium text-foreground">{value}</span>
    </div>
  );
}

function MetricField({
  label,
  value,
  suffix,
  onChange,
}: {
  label: string;
  value: number;
  suffix?: string;
  onChange: (value: number) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-xl border border-border/70 bg-background/80 px-3 py-2.5">
      <div className="text-xs text-foreground">{label}</div>
      <div className="flex items-center gap-2">
        <NumberStepInput
          value={value}
          onChange={onChange}
          decreaseLabel={`减少${label}`}
          increaseLabel={`增加${label}`}
        />
        {suffix ? <span className="text-xs text-muted-foreground">{suffix}</span> : null}
      </div>
    </div>
  );
}

function StrategySwitch({
  label,
  description,
  checked,
  onCheckedChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onCheckedChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-xl border border-border/70 bg-background/80 px-3 py-3">
      <div className="space-y-1">
        <div className="text-xs font-medium text-foreground">{label}</div>
        <div className="text-[11px] leading-5 text-muted-foreground">{description}</div>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} className="mt-0.5 scale-75" />
    </div>
  );
}

export function TaskSettingsForm({ config, onConfigChange }: TaskSettingsFormProps) {
  const [envDialogOpen, setEnvDialogOpen] = useState(false);
  const [selectedEnvNames, setSelectedEnvNames] = useState<Map<string, string>>(new Map());
  const [tagInputOpen, setTagInputOpen] = useState(false);
  const [tagDraft, setTagDraft] = useState('');
  const [advancedOpen, setAdvancedOpen] = useState('');

  useEffect(() => {
    const syncEnvNames = async () => {
      if (config.selectedEnvironments.length === 0) {
        setSelectedEnvNames(new Map());
        return;
      }

      try {
        const envs = (await listRpaEnvironments()) as Environment[];
        const nameMap = new Map<string, string>();
        config.selectedEnvironments.forEach((id) => {
          const env = envs.find((item) => item.id === id);
          if (env) {
            nameMap.set(id, env.name);
          }
        });
        setSelectedEnvNames(nameMap);
      } catch (error) {
        console.error('Failed to fetch environment names:', error);
      }
    };

    void syncEnvNames();
  }, [config.selectedEnvironments]);

  const handleChange = <K extends keyof TaskConfig>(key: K, value: TaskConfig[K]) => {
    onConfigChange({ ...config, [key]: value });
  };

  const handleEnvironmentsConfirm = (selectedIds: string[]) => {
    handleChange('selectedEnvironments', selectedIds);
  };

  const removeEnvironment = (envId: string) => {
    handleChange(
      'selectedEnvironments',
      config.selectedEnvironments.filter((id) => id !== envId)
    );
  };

  const addTag = () => {
    const value = tagDraft.trim();
    if (!value) {
      setTagDraft('');
      setTagInputOpen(false);
      return;
    }

    if (!config.tags.includes(value)) {
      handleChange('tags', [...config.tags, value]);
    }

    setTagDraft('');
    setTagInputOpen(false);
  };

  const removeTag = (tag: string) => {
    handleChange(
      'tags',
      config.tags.filter((item) => item !== tag)
    );
  };

  const triggerLabel = config.triggerType === 'scheduled' ? '定时触发' : '手动触发';
  const triggerDetail =
    config.triggerType === 'scheduled'
      ? config.schedule === 'custom'
        ? config.cronExpression?.trim()
          ? `定时触发 / ${config.cronExpression.trim()}`
          : '定时触发 / 自定义 Cron'
        : config.schedule
          ? `定时触发 / ${
              config.schedule === 'hourly'
                ? '每小时'
                : config.schedule === 'daily'
                  ? '每天'
                  : config.schedule === 'weekly'
                    ? '每周'
                    : '自定义'
            }`
          : '定时触发'
      : '手动触发';
  const strategySummary = [
    triggerLabel,
    `超时 ${config.timeout}s`,
    `重试 ${config.retryCount} 次`,
    config.stopOnError ? '出错即停止' : '允许继续执行',
    config.closeBrowserOnComplete ? '结束后关闭浏览器' : '结束后保留浏览器',
  ].join(' · ');

  return (
    <>
      <ScrollArea className="h-full">
        <div className="space-y-4 pr-2">
          <section className="rounded-2xl border border-border/60 bg-gradient-to-br from-background via-background to-muted/20 p-4">
            <div className="flex flex-wrap gap-2">
              <SummaryBadge
                icon={Workflow}
                label="绑定环境"
                value={`${config.selectedEnvironments.length} 个`}
              />
              <SummaryBadge icon={BadgeCheck} label="默认运行" value="顺序执行" />
            </div>

            <div className="mt-4 px-1 py-1">
              <div className="space-y-1">
                <div className="text-[11px] uppercase tracking-[0.18em] text-muted-foreground/80">
                  基础信息
                </div>
                <Input
                  value={config.name}
                  onChange={(e) => handleChange('name', e.target.value)}
                  placeholder="输入任务名称"
                  className="h-10 border-0 bg-transparent px-0 text-xl font-semibold shadow-none focus-visible:ring-0"
                />
                <Input
                  value={config.description}
                  onChange={(e) => handleChange('description', e.target.value)}
                  placeholder="补充一行简短描述"
                  className="h-8 border-0 bg-transparent px-0 text-sm text-muted-foreground shadow-none focus-visible:ring-0"
                />
              </div>

              <div className="mt-4 space-y-4">
                <div className="space-y-2">
                  <div className="flex items-center justify-between gap-2 text-[11px] text-muted-foreground">
                    <span>任务标签</span>
                    <button
                      type="button"
                      onClick={() => setTagInputOpen(true)}
                      className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-dashed border-border text-muted-foreground transition-colors hover:border-primary/50 hover:text-foreground"
                      aria-label="新增标签"
                    >
                      <Plus className="h-3.5 w-3.5" />
                    </button>
                  </div>
                  {tagInputOpen ? (
                    <div className="flex min-h-11 items-center gap-2 rounded-xl border border-border/60 bg-background px-2.5 py-1.5">
                      <Input
                        value={tagDraft}
                        onChange={(e) => setTagDraft(e.target.value)}
                        onBlur={() => {
                          setTagDraft('');
                          setTagInputOpen(false);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault();
                            addTag();
                          }
                          if (e.key === 'Escape') {
                            setTagDraft('');
                            setTagInputOpen(false);
                          }
                        }}
                        placeholder="输入标签后回车"
                        className="h-8 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
                        autoFocus
                      />
                      <Button
                        type="button"
                        size="sm"
                        className="h-8 text-xs"
                        onMouseDown={(e) => e.preventDefault()}
                        onClick={addTag}
                      >
                        添加
                      </Button>
                    </div>
                  ) : (
                    <div className="flex min-h-11 flex-wrap items-center gap-1.5 rounded-xl border border-border/60 bg-background px-2.5 py-2">
                      {config.tags.length > 0 ? (
                        config.tags.map((tag) => (
                          <Badge key={tag} variant="secondary" className="gap-1 px-2 py-0.5 text-[10px]">
                            <span>{tag}</span>
                            <button
                              type="button"
                              className="inline-flex h-3.5 w-3.5 items-center justify-center rounded-sm hover:text-destructive"
                              onClick={(e) => {
                                e.stopPropagation();
                                removeTag(tag);
                              }}
                              aria-label={`删除标签 ${tag}`}
                            >
                              <X className="h-3 w-3" />
                            </button>
                          </Badge>
                        ))
                      ) : (
                        <span className="text-[11px] text-muted-foreground">当前还没有标签。</span>
                      )}
                    </div>
                  )}
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between gap-2 text-[11px] text-muted-foreground">
                    <span>绑定环境</span>
                    <button
                      type="button"
                      onClick={() => setEnvDialogOpen(true)}
                      className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-dashed border-border text-muted-foreground transition-colors hover:border-primary/50 hover:text-foreground"
                      aria-label="管理环境"
                    >
                      <RefreshCw className="h-3.5 w-3.5" />
                    </button>
                  </div>
                  <div className="flex min-h-11 flex-wrap items-center gap-1.5 rounded-xl border border-border/60 bg-background px-2.5 py-2">
                    {config.selectedEnvironments.length > 0 ? (
                      config.selectedEnvironments.map((envId) => (
                        <Badge key={envId} variant="secondary" className="gap-1 px-2 py-0.5 text-[10px]">
                          <span className="max-w-28 truncate">{selectedEnvNames.get(envId) || envId}</span>
                          <button
                            type="button"
                            className="inline-flex h-3.5 w-3.5 items-center justify-center rounded-sm hover:text-destructive"
                            onClick={(e) => {
                              e.stopPropagation();
                              removeEnvironment(envId);
                            }}
                            aria-label="移除环境"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))
                    ) : (
                      <span className="text-[11px] text-muted-foreground">当前还没有绑定环境。</span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </section>

          <section className="rounded-2xl border border-border/60 bg-muted/20 px-4">
            <Accordion
              type="single"
              collapsible
              value={advancedOpen}
              onValueChange={setAdvancedOpen}
              className="w-full"
            >
              <AccordionItem value="advanced" className="border-b-0">
                <AccordionTrigger className="py-4 hover:no-underline">
                  <div className="space-y-1 text-left">
                    <div className="text-sm font-semibold text-foreground">执行策略</div>
                    <div className="text-[11px] text-muted-foreground">{strategySummary}</div>
                  </div>
                </AccordionTrigger>
                <AccordionContent className="pb-4">
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <div className="text-[11px] text-muted-foreground">触发方式</div>
                      <div
                        className={`grid gap-3 md:items-center ${
                          config.triggerType === 'scheduled'
                            ? 'md:grid-cols-[44px_minmax(0,1fr)_220px]'
                            : 'md:grid-cols-[44px_minmax(0,1fr)]'
                        }`}
                      >
                        <button
                          type="button"
                          onClick={() =>
                            handleChange(
                              'triggerType',
                              config.triggerType === 'scheduled' ? 'manual' : 'scheduled'
                            )
                          }
                          className={`inline-flex h-11 w-11 items-center justify-center rounded-xl border transition-colors ${
                            config.triggerType === 'scheduled'
                              ? 'border-primary bg-primary/8 text-primary'
                              : 'border-border bg-background text-muted-foreground hover:border-primary/40'
                          }`}
                          aria-label="切换触发方式"
                        >
                          {config.triggerType === 'scheduled' ? (
                            <Clock3 className="h-4 w-4" />
                          ) : (
                            <Hand className="h-4 w-4" />
                          )}
                        </button>

                        <div className="flex h-11 items-center rounded-xl border border-border/60 bg-background px-3 text-sm text-foreground">
                          {triggerDetail}
                        </div>

                        {config.triggerType === 'scheduled' ? (
                          <div
                            className={`grid gap-2 ${
                              config.schedule === 'custom' ? 'md:grid-cols-2' : 'md:grid-cols-1'
                            }`}
                          >
                            <Select
                              value={config.schedule || ''}
                              onValueChange={(value) => handleChange('schedule', value)}
                            >
                              <SelectTrigger className="h-11! w-full rounded-xl border-border/60 bg-background text-xs shadow-none">
                                <SelectValue placeholder="选择计划" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="hourly">每小时</SelectItem>
                                <SelectItem value="daily">每天</SelectItem>
                                <SelectItem value="weekly">每周</SelectItem>
                                <SelectItem value="custom">自定义</SelectItem>
                              </SelectContent>
                            </Select>

                            {config.schedule === 'custom' ? (
                              <Input
                                value={config.cronExpression || ''}
                                onChange={(e) => handleChange('cronExpression', e.target.value)}
                                placeholder="0 0 * * *"
                                className="h-11 border-border/60 bg-background font-mono text-xs"
                              />
                            ) : null}
                          </div>
                        ) : null}
                      </div>
                    </div>

                    <div className="grid gap-2">
                      <MetricField
                        label="重试次数"
                        value={config.retryCount}
                        onChange={(value) => handleChange('retryCount', value)}
                      />
                      <MetricField
                        label="重试间隔"
                        value={config.retryInterval}
                        suffix="秒"
                        onChange={(value) => handleChange('retryInterval', value)}
                      />
                      <MetricField
                        label="超时时间"
                        value={config.timeout}
                        suffix="秒"
                        onChange={(value) => handleChange('timeout', value)}
                      />
                    </div>

                    <div className="grid gap-2">
                      <StrategySwitch
                        label="出错时停止"
                        description="流程遇到错误后立即停止，避免继续执行后续步骤。"
                        checked={config.stopOnError}
                        onCheckedChange={(value) => handleChange('stopOnError', value)}
                      />
                      <StrategySwitch
                        label="结束后关闭浏览器"
                        description="仅影响正式执行。启用后，任务执行完成会自动关闭本次启动的浏览器。"
                        checked={config.closeBrowserOnComplete}
                        onCheckedChange={(value) => handleChange('closeBrowserOnComplete', value)}
                      />
                      <StrategySwitch
                        label="完成时通知"
                        description="任务完成后发送通知。"
                        checked={config.notifyOnComplete}
                        onCheckedChange={(value) => handleChange('notifyOnComplete', value)}
                      />
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </section>
        </div>
      </ScrollArea>

      <EnvironmentSelectorDialog
        open={envDialogOpen}
        onOpenChange={setEnvDialogOpen}
        selectedIds={config.selectedEnvironments}
        onConfirm={handleEnvironmentsConfirm}
      />
    </>
  );
}
