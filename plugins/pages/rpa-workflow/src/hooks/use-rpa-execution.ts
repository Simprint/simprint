import { useCallback, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  extractTaskRunOptions,
  extractWorkflowSchema,
  getRpaTaskDetail,
  type RpaTaskDetailDto,
} from '../api';
import type { RpaTask, RpaWorkflowStep } from '../types';
import { RpaRunner } from '../runtime/runner';
import { CdpBrowserAdapter } from '../runtime/cdp-browser-adapter';
import { getEnvironmentCdpEndpoint, getEnvironmentStatus } from '../runtime/tauri';
import {
  getEnvironmentDetail,
  startEnvironmentRuntime,
  stopEnvironmentRuntime,
} from '../../../../services/environment/src';

export type RpaFormalExecutionStatus = 'running' | 'stopping' | 'success' | 'failed' | 'stopped';
export type RpaFormalExecutionEnvStatus =
  | 'pending'
  | 'starting'
  | 'running'
  | 'awaiting_input'
  | 'success'
  | 'failed'
  | 'stopped';

export interface RpaExecutionEnvironmentItem {
  envUuid: string;
  name: string;
  status: RpaFormalExecutionEnvStatus;
  currentStepId?: string;
  currentStepName?: string;
  error?: string;
  startedAt?: string;
  finishedAt?: string;
  completedStepIds?: string[];
  failedStepId?: string;
}

export interface RpaExecutionState {
  taskId: string;
  taskName: string;
  status: RpaFormalExecutionStatus;
  startedAt: string;
  finishedAt?: string;
  currentEnvUuid?: string;
  currentStepId?: string;
  currentStepName?: string;
  totalSteps: number;
  workflow: {
    startStepId: string | null;
    steps: RpaWorkflowStep[];
  };
  environmentRuns: RpaExecutionEnvironmentItem[];
}

interface PendingExecution {
  task: RpaTask;
  detail: RpaTaskDetailDto;
}

interface UseRpaExecutionParams {
  tasks: RpaTask[];
}

interface UseRpaExecutionResult {
  runDialog: {
    open: boolean;
    loading: boolean;
    pending: PendingExecution | null;
    openDialog: (id: string) => Promise<void>;
    closeDialog: () => void;
    confirmRun: (variableOverrides?: Record<string, string>) => Promise<void>;
  };
  executionDrawer: {
    open: boolean;
    execution: RpaExecutionState | null;
    openDrawer: () => void;
    openDrawerForTask: (taskId: string) => void;
    closeDrawer: () => void;
    stopExecution: () => Promise<void>;
  };
  handleRun: (id: string) => Promise<void>;
  handleStop: (id: string) => Promise<void>;
  isTaskExecuting: (id: string) => boolean;
  isTaskStopping: (id: string) => boolean;
  canViewExecution: (id: string) => boolean;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function normalizeExecutionError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);

  if (message === 'OPEN_PAGE_REQUIRED' || message.includes('OPEN_PAGE_REQUIRED')) {
    return '请先在该流程前添加“打开新页面”步骤。';
  }

  if (message === 'CDP_CONNECTION_CLOSED' || message.includes('CDP connection closed')) {
    return '浏览器连接已关闭，请重新执行。';
  }

  if (message === 'TAB_INDEX_INVALID' || message.includes('TAB_INDEX_INVALID')) {
    return '请选择有效的标签页位置。';
  }

  if (message.startsWith('TAB_INDEX_OUT_OF_RANGE:')) {
    const [, rawIndex, rawTotal] = message.split(':');
    return `标签页位置超出范围。当前共有 ${rawTotal || '?'} 个标签页，找不到第 ${rawIndex || '?'} 个标签页。`;
  }

  if (message === 'TAB_CLOSE_LAST_UNSUPPORTED' || message.includes('TAB_CLOSE_LAST_UNSUPPORTED')) {
    return '至少需要保留一个标签页，无法关闭最后一个标签页。';
  }

  if (message === 'NO_ACTIVE_TAB_AFTER_CLOSE' || message.includes('NO_ACTIVE_TAB_AFTER_CLOSE')) {
    return '关闭标签页后未找到可用的活动标签页，请重新执行。';
  }

  if (message === 'BREAK_LOOP_OUTSIDE_LOOP' || message.includes('BREAK_LOOP_OUTSIDE_LOOP')) {
    return '退出循环节点只能在循环区域内使用。';
  }

  if (message === 'NODE_RUNTIME_NOT_FOUND' || message.includes('NODE_RUNTIME_NOT_FOUND')) {
    return '本地执行需要 Node.js 环境，但当前系统未检测到 Node。';
  }

  if (message === 'LOCAL_SCRIPT_TIMEOUT' || message.includes('LOCAL_SCRIPT_TIMEOUT')) {
    return '本地脚本执行超时。';
  }

  if (message === 'TAB_TARGET_UNAVAILABLE' || message === 'RPA_BROWSER_NOT_FOUND') {
    return '当前标签页无法连接，请重新执行。';
  }

  if (message === 'LOOP_BODY_CYCLE') {
    return '循环体内部存在无法结束的回路。';
  }

  return message;
}

async function waitForCdpEndpoint(envUuid: string, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const endpoint = await getEnvironmentCdpEndpoint(envUuid);
    if (endpoint?.browser_ws_url) {
      return endpoint;
    }
    await sleep(500);
  }

  throw new Error('浏览器调试连接未及时就绪');
}

export function useRpaExecution(params: UseRpaExecutionParams): UseRpaExecutionResult {
  const { tasks } = params;
  const [runDialogOpen, setRunDialogOpen] = useState(false);
  const [runDialogLoading, setRunDialogLoading] = useState(false);
  const [pendingExecution, setPendingExecution] = useState<PendingExecution | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [execution, setExecution] = useState<RpaExecutionState | null>(null);
  const stopRequestedRef = useRef(false);

  const taskMap = useMemo(() => new Map(tasks.map((task) => [task.id, task])), [tasks]);

  const updateEnvironmentRun = useCallback(
    (envUuid: string, updater: (item: RpaExecutionEnvironmentItem) => RpaExecutionEnvironmentItem) => {
      setExecution((current) => {
        if (!current) {
          return current;
        }

        return {
          ...current,
          environmentRuns: current.environmentRuns.map((item) =>
            item.envUuid === envUuid ? updater(item) : item
          ),
        };
      });
    },
    []
  );

  const openDialog = useCallback(
    async (id: string) => {
      const task = taskMap.get(id);
      if (!task) {
        return;
      }

      setRunDialogLoading(true);
      try {
        const detail = await getRpaTaskDetail(id);
        setPendingExecution({ task, detail });
        setRunDialogOpen(true);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : '加载任务详情失败');
      } finally {
        setRunDialogLoading(false);
      }
    },
    [taskMap]
  );

  const closeDialog = useCallback(() => {
    setRunDialogOpen(false);
    window.setTimeout(() => setPendingExecution(null), 0);
  }, []);

  const closeDrawer = useCallback(() => {
    setDrawerOpen(false);
  }, []);

  const openDrawerForTask = useCallback(
    (taskId: string) => {
      if (execution?.taskId === taskId) {
        setDrawerOpen(true);
      }
    },
    [execution?.taskId]
  );

  const confirmRun = useCallback(async (variableOverrides: Record<string, string> = {}) => {
    if (!pendingExecution) {
      return;
    }

    const workflow = extractWorkflowSchema(pendingExecution.detail);
    if (!workflow) {
      toast.error('任务流程数据无效');
      return;
    }
    const runOptions = extractTaskRunOptions(pendingExecution.detail);

    if (!pendingExecution.detail.environment_uuids?.length) {
      toast.error('请先绑定环境后再执行');
      return;
    }

    const workflowWithOverrides = {
      ...workflow,
      variables: workflow.variables.map((variable) => ({
        ...variable,
        value:
          Object.prototype.hasOwnProperty.call(variableOverrides, variable.name)
            ? variableOverrides[variable.name] ?? ''
            : variable.value,
      })),
    };

    stopRequestedRef.current = false;
    const startedAt = new Date().toISOString();
    setExecution({
      taskId: pendingExecution.task.id,
      taskName: pendingExecution.task.name,
      status: 'running',
      startedAt,
      totalSteps: workflowWithOverrides.steps.length,
      workflow: {
        startStepId: workflowWithOverrides.start_step_id,
        steps: workflowWithOverrides.steps,
      },
      currentStepName: undefined,
      currentEnvUuid: undefined,
      environmentRuns: pendingExecution.detail.environment_uuids.map((envUuid) => ({
        envUuid,
        name: envUuid,
        status: 'pending',
        completedStepIds: [],
      })),
    });
    setDrawerOpen(true);
    setRunDialogOpen(false);
    setPendingExecution(null);

    void (async () => {
      for (const envUuid of pendingExecution.detail.environment_uuids) {
        if (stopRequestedRef.current) {
          break;
        }

        let startedByRuntime = false;
        try {
          const environmentDetail = await getEnvironmentDetail(envUuid);
          const envName = environmentDetail.environment?.name?.trim() || envUuid;
          updateEnvironmentRun(envUuid, (item) => ({
            ...item,
            name: envName,
            status: 'starting',
            startedAt: new Date().toISOString(),
            error: undefined,
          }));
          setExecution((current) =>
            current
              ? {
                  ...current,
                  currentEnvUuid: envUuid,
                  currentStepName: undefined,
                }
              : current
          );

          const envStatus = await getEnvironmentStatus(envUuid);
          if (envStatus !== 'running') {
            await startEnvironmentRuntime(envUuid);
            startedByRuntime = true;
          }

          const endpoint = await waitForCdpEndpoint(envUuid);
          const runner = new RpaRunner(new CdpBrowserAdapter());
          const stepNameMap = new Map(
            workflowWithOverrides.steps.map((step) => [step.id, step.name])
          );

          await runner.run(
            workflowWithOverrides,
            { envUuid, browserWsUrl: endpoint.browser_ws_url! },
            {
              onStepStatusChange: (event) => {
                updateEnvironmentRun(envUuid, (item) => ({
                  ...item,
                  status:
                    event.status === 'failed'
                      ? 'failed'
                      : event.status === 'success'
                        ? 'running'
                      : event.status === 'awaiting_input'
                        ? 'awaiting_input'
                        : 'running',
                  currentStepId: event.stepId,
                  currentStepName: stepNameMap.get(event.stepId) || event.stepType,
                  completedStepIds:
                    event.status === 'success'
                      ? Array.from(new Set([...(item.completedStepIds ?? []), event.stepId]))
                      : item.completedStepIds ?? [],
                  failedStepId: event.status === 'failed' ? event.stepId : item.failedStepId,
                  error: event.status === 'failed' ? normalizeExecutionError(event.error) : undefined,
                }));
                setExecution((current) =>
                  current
                    ? {
                        ...current,
                        currentEnvUuid: envUuid,
                        currentStepId: event.stepId,
                        currentStepName: stepNameMap.get(event.stepId) || event.stepType,
                      }
                    : current
                );
              },
            }
          );

          updateEnvironmentRun(envUuid, (item) => ({
            ...item,
            status: 'success',
            finishedAt: new Date().toISOString(),
          }));
        } catch (error) {
          const failedBecauseStopped = stopRequestedRef.current;
          const normalizedError = normalizeExecutionError(error);
          updateEnvironmentRun(envUuid, (item) => ({
            ...item,
            status: failedBecauseStopped ? 'stopped' : 'failed',
            error: failedBecauseStopped ? undefined : normalizedError,
            finishedAt: new Date().toISOString(),
          }));

          if (!failedBecauseStopped && workflowWithOverrides.settings.stop_on_error) {
            break;
          }
        } finally {
          if (startedByRuntime && runOptions.closeBrowserOnComplete) {
            await stopEnvironmentRuntime(envUuid).catch(() => undefined);
          }
        }
      }

      setExecution((current) => {
        if (!current) {
          return current;
        }

        const hasFailed = current.environmentRuns.some((item) => item.status === 'failed');
        const hasStopped =
          stopRequestedRef.current || current.environmentRuns.some((item) => item.status === 'stopped');
        return {
          ...current,
          status: hasStopped ? 'stopped' : hasFailed ? 'failed' : 'success',
          finishedAt: new Date().toISOString(),
        };
      });
    })();
  }, [pendingExecution, updateEnvironmentRun]);

  const stopExecution = useCallback(async () => {
    stopRequestedRef.current = true;
    const currentEnvUuid = execution?.currentEnvUuid;
    setExecution((current) => (current ? { ...current, status: 'stopping' } : current));

    if (currentEnvUuid) {
      await stopEnvironmentRuntime(currentEnvUuid).catch(() => undefined);
      updateEnvironmentRun(currentEnvUuid, (item) => ({
        ...item,
        status: 'stopped',
        finishedAt: new Date().toISOString(),
      }));
    }
  }, [execution?.currentEnvUuid, updateEnvironmentRun]);

  const handleStop = useCallback(
    async (id: string) => {
      if (execution?.taskId !== id) {
        return;
      }
      setDrawerOpen(true);
      await stopExecution();
    },
    [execution?.taskId, stopExecution]
  );

  const isTaskExecuting = useCallback(
    (id: string) => execution?.taskId === id && (execution.status === 'running' || execution.status === 'stopping'),
    [execution]
  );

  const isTaskStopping = useCallback(
    (id: string) => execution?.taskId === id && execution.status === 'stopping',
    [execution]
  );

  const canViewExecution = useCallback((id: string) => execution?.taskId === id, [execution?.taskId]);

  return {
    runDialog: {
      open: runDialogOpen,
      loading: runDialogLoading,
      pending: pendingExecution,
      openDialog,
      closeDialog,
      confirmRun,
    },
    executionDrawer: {
      open: drawerOpen,
      execution,
      openDrawer: () => setDrawerOpen(true),
      openDrawerForTask,
      closeDrawer,
      stopExecution,
    },
    handleRun: openDialog,
    handleStop,
    isTaskExecuting,
    isTaskStopping,
    canViewExecution,
  };
}
