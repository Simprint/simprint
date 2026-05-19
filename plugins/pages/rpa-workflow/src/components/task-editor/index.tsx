import { useTranslation } from 'react-i18next';
import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import {
  ArrowLeft,
  Save,
  Play,
  Square,
  Shield,
  Settings,
  Settings2,
} from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion';
import { FormattedDialog, FormattedDialogFooter } from '@/components/formatted-dialog';
import { cn } from '@/lib/utils';
import { createRpaTask, getRpaTaskDetail, updateRpaTask } from '../../api';
import {
  startAnonymousRpaEnvironment,
  stopAnonymousRpaEnvironment,
} from '../../runtime/anonymous-environment';
import { CdpBrowserAdapter } from '../../runtime/cdp-browser-adapter';
import {
  ENV_CONNECTION_STATUS_EVENT,
  getEnvironmentStatus,
  type EnvConnectionPayload,
  type EnvironmentStatus,
} from '../../runtime/tauri';
import {
  RpaRunner,
  type RpaRunnerCapturedTargetEvent,
  type RpaRunnerExtractedDataEvent,
  type RpaRunnerScreenshotEvent,
  type RpaRunnerStepEvent,
  type RpaRunnerStepStatus,
} from '../../runtime/runner';
import { ComponentPanel, type ComponentItem } from './component-panel';
import {
  DEFAULT_SPECIAL_NODE_POSITIONS,
  FlowCanvas,
  type FlowStep,
  type SpecialNodePositions,
} from './flow-canvas';
import { getLoopAutoDimensions, getLoopAutoPosition } from './loop-layout';
import { PropertyPanel } from './property-panel';
import { TaskSettingsForm, type TaskConfig, type TaskVariable } from './task-settings-form';
import { VariablePanel, type RuntimeVariableItem } from './variable-panel';
import {
  AnonymousProxyDrawer,
  type AnonymousProxyCandidate,
} from './anonymous-proxy-drawer';
import { buildTaskPayload, buildWorkflowSchema, mapPortableTaskToEditorState, mapTaskDetailToEditorState } from './workflow-mapper';
import type { PortableRpaTaskDocument } from '../../lib/rpa-transfer';
import type { ProxyConfig } from '../../../../services/environment/src';
import type { RpaWorkflowSchema } from '../../types';

function createDefaultTaskConfig(): TaskConfig {
  const now = new Date();
  const stamp = `${now.getMonth() + 1}${now.getDate()}-${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}`;

  return {
    name: `RPA任务-${stamp}`,
    description: '自动化流程',
    tags: [],
    triggerType: 'manual',
    selectedEnvironments: [],
    runMode: 'sequential',
    globalVariables: [],
    retryCount: 3,
    retryInterval: 5,
    timeout: 300,
    concurrency: 1,
    stopOnError: true,
    notifyOnComplete: false,
    notifyOnError: false,
  };
}

const DEFAULT_NAVIGATE_URL = 'https://www.google.com';
const DEFAULT_INPUT_TEXT = 'Hello World';

function createDefaultExtractVariableName() {
  return `extract_${Math.random().toString(36).slice(2, 8)}`;
}

function createDefaultScriptVariableName() {
  return `script_${Math.random().toString(36).slice(2, 8)}`;
}

function findInvalidBreakLoopStep(steps: FlowStep[]): FlowStep | null {
  const loopIds = new Set(steps.filter((step) => step.type === 'loop').map((step) => step.id));
  return (
    steps.find(
      (step) =>
        step.type === 'break_loop' &&
        (!step.parentLoopId || !loopIds.has(step.parentLoopId))
    ) ?? null
  );
}

type RunStatus = 'idle' | 'starting' | 'running' | 'success' | 'failed' | 'stopping' | 'stopped';

interface RunStepItem {
  id: string;
  name: string;
  type: string;
  status: RpaRunnerStepStatus;
  error?: string;
  loopIteration?: number;
  loopTotal?: number;
  incomingFromStepId?: string;
  incomingBranchKey?: 'true' | 'false';
}

interface EditorHistorySnapshot {
  steps: FlowStep[];
  specialPositions: SpecialNodePositions;
  selectedStepId: string | null;
}

interface EditorHistoryState {
  past: EditorHistorySnapshot[];
  future: EditorHistorySnapshot[];
}

const MAX_EDITOR_HISTORY = 100;

function cloneHistorySnapshot(snapshot: EditorHistorySnapshot): EditorHistorySnapshot {
  return JSON.parse(JSON.stringify(snapshot)) as EditorHistorySnapshot;
}

function normalizeHistorySnapshot(snapshot: EditorHistorySnapshot): EditorHistorySnapshot {
  const selectedStepId =
    snapshot.selectedStepId && snapshot.steps.some((step) => step.id === snapshot.selectedStepId)
      ? snapshot.selectedStepId
      : null;

  return {
    steps: snapshot.steps,
    specialPositions: snapshot.specialPositions,
    selectedStepId,
  };
}

function areHistorySnapshotsEqual(a: EditorHistorySnapshot, b: EditorHistorySnapshot): boolean {
  return JSON.stringify(normalizeHistorySnapshot(a)) === JSON.stringify(normalizeHistorySnapshot(b));
}

function isEditableUndoTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable) {
    return true;
  }

  const tagName = target.tagName.toLowerCase();
  return tagName === 'input' || tagName === 'textarea' || tagName === 'select';
}

function buildAnonymousProxyConfig(
  proxy: AnonymousProxyCandidate | null
): ProxyConfig | null {
  if (!proxy?.host || !proxy.port) {
    return null;
  }

  return {
    host: proxy.host,
    port: proxy.port,
    proxy_type: proxy.proxy_type || 'http',
    username: proxy.username || undefined,
    password: proxy.password
      ? {
          value: proxy.password,
          encrypted: false,
        }
      : undefined,
  };
}

function getAnonymousProxyLabel(
  proxy: AnonymousProxyCandidate | null,
  t: ReturnType<typeof useTranslation>['t']
): string {
  if (!proxy) {
    return t('editor.proxy.none', { defaultValue: '未配置代理' });
  }

  return proxy.source === 'local'
    ? proxy.name
    : `${proxy.host}:${proxy.port}`;
}

export function TaskEditor() {
  const { t } = useTranslation('rpa');
  const navigate = useNavigate();
  const location = useLocation();
  const { id } = useParams<{ id: string }>();
  const isEditing = Boolean(id);
  const importedDocument = (location.state as { importedRpaDocument?: PortableRpaTaskDocument } | null)?.importedRpaDocument ?? null;

  const [config, setConfig] = useState<TaskConfig>(() => createDefaultTaskConfig());
  const [steps, setSteps] = useState<FlowStep[]>([]);
  const [specialPositions, setSpecialPositions] = useState<SpecialNodePositions>(
    DEFAULT_SPECIAL_NODE_POSITIONS
  );
  const [selectedStepId, setSelectedStepId] = useState<string | null>(null);
  const [selectedComponent, setSelectedComponent] = useState<ComponentItem | null>(null);
  const [saving, setSaving] = useState(false);
  const [loadingTask, setLoadingTask] = useState(isEditing);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [sidebarSection, setSidebarSection] = useState<'properties' | 'variables' | ''>('properties');
  const [running, setRunning] = useState(false);
  const [runStatus, setRunStatus] = useState<RunStatus>('idle');
  const [runStepItems, setRunStepItems] = useState<RunStepItem[]>([]);
  const [activeRunEnvUuid, setActiveRunEnvUuid] = useState<string | null>(null);
  const [activeRunEnvStatus, setActiveRunEnvStatus] = useState<EnvironmentStatus | undefined>(undefined);
  const [proxyDrawerOpen, setProxyDrawerOpen] = useState(false);
  const [selectedAnonymousProxy, setSelectedAnonymousProxy] =
    useState<AnonymousProxyCandidate | null>(null);
  const stopRequestedRef = useRef(false);
  const editorSnapshotRef = useRef<EditorHistorySnapshot>({
    steps: [],
    specialPositions: DEFAULT_SPECIAL_NODE_POSITIONS,
    selectedStepId: null,
  });
  const [editorHistory, setEditorHistory] = useState<EditorHistoryState>({
    past: [],
    future: [],
  });

  const selectedStep = steps.find((s) => s.id === selectedStepId) || null;
  const stepStatusMap = useMemo(
    () => Object.fromEntries(runStepItems.map((step) => [step.id, step.status])),
    [runStepItems]
  );
  const stepErrorMap = useMemo(
    () => Object.fromEntries(runStepItems.map((step) => [step.id, step.error ?? ''])),
    [runStepItems]
  );
  const loopProgressMap = useMemo(
    () =>
      Object.fromEntries(
        runStepItems.map((step) => [
          step.id,
          {
            current: step.loopIteration,
            total: step.loopTotal,
          },
        ])
      ),
    [runStepItems]
  );
  const stepIncomingEdgeMap = useMemo(
    () =>
      Object.fromEntries(
        runStepItems.map((step) => [
          step.id,
          {
            fromStepId: step.incomingFromStepId,
            branchKey: step.incomingBranchKey,
          },
        ])
      ),
    [runStepItems]
  );
  const activeStepId = useMemo(
    () =>
      runStepItems.find(
        (step) => step.status === 'running' || step.status === 'awaiting_input'
      )?.id ?? null,
    [runStepItems]
  );
  const runtimeVariables = useMemo<RuntimeVariableItem[]>(() =>
    steps
      .map((step) => {
        const name = typeof step.config.outputKey === 'string' ? step.config.outputKey.trim() : '';
        if (!name || (step.type !== 'extract' && step.type !== 'script')) {
          return null;
        }

        const value = typeof step.config.previewValue === 'string' ? step.config.previewValue : '';
        if (!value) {
          return null;
        }

        return {
          name,
          value,
          source: step.type === 'script' ? 'script' : 'extract',
        } satisfies RuntimeVariableItem;
      })
      .filter((variable): variable is RuntimeVariableItem => Boolean(variable)),
    [steps]
  );

  useEffect(() => {
    setSteps((prev) => {
      let changed = false;
      const next = prev.map((step) => {
        if (step.type !== 'extract' && step.type !== 'script') {
          return step;
        }

        const currentOutputKey =
          typeof step.config.outputKey === 'string' ? step.config.outputKey.trim() : '';
        if (currentOutputKey) {
          return step;
        }

        changed = true;
        return {
          ...step,
          config: {
            ...step.config,
            outputKey:
              step.type === 'script'
                ? createDefaultScriptVariableName()
                : createDefaultExtractVariableName(),
          },
        };
      });

      return changed ? next : prev;
    });
  }, []);
  const hasActiveRunEnvironment =
    Boolean(activeRunEnvUuid) &&
    activeRunEnvStatus !== 'stopped';
  const runButtonBusy = running || hasActiveRunEnvironment || runStatus === 'stopping';
  const proxyButtonDisabled =
    runStatus === 'starting' || runStatus === 'running' || runStatus === 'stopping';
  const anonymousProxyLabel = useMemo(
    () => getAnonymousProxyLabel(selectedAnonymousProxy, t),
    [selectedAnonymousProxy, t]
  );
  const anonymousProxyConfig = useMemo(
    () => buildAnonymousProxyConfig(selectedAnonymousProxy),
    [selectedAnonymousProxy]
  );
  const canUndo = editorHistory.past.length > 0 && !runButtonBusy;

  useEffect(() => {
    editorSnapshotRef.current = {
      steps,
      specialPositions,
      selectedStepId,
    };
  }, [steps, specialPositions, selectedStepId]);

  const applyEditorSnapshot = useCallback((snapshot: EditorHistorySnapshot) => {
    const normalizedSnapshot = normalizeHistorySnapshot(cloneHistorySnapshot(snapshot));
    setSteps(normalizedSnapshot.steps);
    setSpecialPositions(normalizedSnapshot.specialPositions);
    setSelectedStepId(normalizedSnapshot.selectedStepId);
    setSelectedComponent(null);
  }, []);

  const commitEditorChange = useCallback(
    (updater: (current: EditorHistorySnapshot) => EditorHistorySnapshot | null) => {
      const currentSnapshot = normalizeHistorySnapshot(
        cloneHistorySnapshot(editorSnapshotRef.current)
      );
      const nextSnapshot = updater(currentSnapshot);
      if (!nextSnapshot) {
        return;
      }

      const normalizedNextSnapshot = normalizeHistorySnapshot(
        cloneHistorySnapshot(nextSnapshot)
      );
      if (areHistorySnapshotsEqual(currentSnapshot, normalizedNextSnapshot)) {
        return;
      }

      setEditorHistory((prev) => ({
        past: [...prev.past.slice(-(MAX_EDITOR_HISTORY - 1)), currentSnapshot],
        future: [],
      }));
      applyEditorSnapshot(normalizedNextSnapshot);
    },
    [applyEditorSnapshot]
  );

  const resetEditorHistory = useCallback((snapshot: EditorHistorySnapshot) => {
    setEditorHistory({ past: [], future: [] });
    applyEditorSnapshot(snapshot);
  }, [applyEditorSnapshot]);

  const handleUndo = useCallback(() => {
    if (!canUndo) {
      return;
    }

    const previousSnapshot = editorHistory.past[editorHistory.past.length - 1];
    if (!previousSnapshot) {
      return;
    }

    const currentSnapshot = normalizeHistorySnapshot(
      cloneHistorySnapshot(editorSnapshotRef.current)
    );

    applyEditorSnapshot(previousSnapshot);
    setEditorHistory((prev) => ({
      past: prev.past.slice(0, -1),
      future: [currentSnapshot, ...prev.future].slice(0, MAX_EDITOR_HISTORY),
    }));
  }, [applyEditorSnapshot, canUndo, editorHistory]);
  const normalizeRunError = useCallback(
    (error?: string) => {
      if (!error) {
        return undefined;
      }

      if (
        error === 'OPEN_PAGE_REQUIRED' ||
        error === 'Please add an "Open New Page" step before this click step.'
      ) {
        return t('editor.errors.openPageRequired');
      }

      if (error === 'CDP_CONNECTION_CLOSED') {
        return t('editor.errors.browserConnectionClosed', { defaultValue: '浏览器连接已关闭，请重新运行。' });
      }

      if (error === 'TAB_INDEX_INVALID') {
        return t('editor.errors.tabIndexInvalid', { defaultValue: '请选择有效的标签页位置。' });
      }

      if (error.startsWith('TAB_INDEX_OUT_OF_RANGE:')) {
        const [, rawIndex, rawTotal] = error.split(':');
        const index = Number(rawIndex);
        const total = Number(rawTotal);
        return t('editor.errors.tabIndexOutOfRange', {
          defaultValue: '标签页位置超出范围。当前共有 {{total}} 个标签页，找不到第 {{index}} 个标签页。',
          index: Number.isFinite(index) ? index : rawIndex,
          total: Number.isFinite(total) ? total : rawTotal,
        });
      }

      if (error === 'TAB_CLOSE_LAST_UNSUPPORTED') {
        return t('editor.errors.tabCloseLastUnsupported', {
          defaultValue: '至少需要保留一个标签页，无法关闭最后一个标签页。',
        });
      }

      if (error === 'NO_ACTIVE_TAB_AFTER_CLOSE') {
        return t('editor.errors.noActiveTabAfterClose', {
          defaultValue: '关闭标签页后未找到可用的活动标签页，请重新运行。',
        });
      }

      if (error === 'BREAK_LOOP_OUTSIDE_LOOP') {
        return t('editor.errors.breakLoopOutside', {
          defaultValue: '退出循环节点只能在循环区域内使用。',
        });
      }

      if (error === 'NODE_RUNTIME_NOT_FOUND') {
        return t('editor.errors.nodeRuntimeNotFound', {
          defaultValue: '本地执行需要 Node.js 环境，但当前系统未检测到 Node。',
        });
      }

      if (error === 'LOCAL_SCRIPT_TIMEOUT') {
        return t('editor.errors.localScriptTimeout', {
          defaultValue: '本地脚本执行超时。',
        });
      }

      if (error === 'TAB_TARGET_UNAVAILABLE' || error === 'RPA_BROWSER_NOT_FOUND') {
        return t('editor.errors.browserConnectionClosed', { defaultValue: '浏览器连接已关闭，请重新运行。' });
      }

      if (error === 'CONDITION_TEXT_REQUIRED') {
        return t('editor.condition.missingText', { defaultValue: '请先设置目标文本' });
      }

      if (error === 'CONDITION_URL_REQUIRED') {
        return t('editor.condition.missingUrl', { defaultValue: '请先设置目标地址' });
      }

      if (
        error.startsWith('Failed to click target:') ||
        error.startsWith('Failed to fill target:') ||
        error.startsWith('Timed out waiting for target:') ||
        error.startsWith('Failed to resolve screenshot area:')
      ) {
        return t('editor.errors.selectorMissing', { defaultValue: '目标元素找不到或已失效，请清除后重新选择。' });
      }

      return error;
    },
    [t]
  );

  useEffect(() => {
    if (!id) {
      setLoadingTask(false);
      return;
    }

    let cancelled = false;

    const loadTask = async () => {
      setLoadingTask(true);
      try {
        const detail = await getRpaTaskDetail(id);
        if (cancelled) {
          return;
        }

        const editorState = mapTaskDetailToEditorState(detail);
        setConfig(editorState.config);
        resetEditorHistory({
          steps: editorState.steps,
          specialPositions: editorState.specialPositions,
          selectedStepId: null,
        });
      } catch (e) {
        if (!cancelled) {
          toast.error(e instanceof Error ? e.message : 'Failed to load task.');
          navigate('/rpa');
        }
      } finally {
        if (!cancelled) {
          setLoadingTask(false);
        }
      }
    };

    void loadTask();

    return () => {
      cancelled = true;
    };
  }, [id, navigate, resetEditorHistory]);

  useEffect(() => {
    if (isEditing) {
      return;
    }

    setConfig((prev) => {
      if (prev.name.trim() && prev.description.trim()) {
        return prev;
      }

      const fallback = createDefaultTaskConfig();
      return {
        ...prev,
        name: prev.name.trim() ? prev.name : fallback.name,
        description: prev.description.trim() ? prev.description : fallback.description,
      };
    });
  }, [isEditing]);
  useEffect(() => {
    if (isEditing || !importedDocument) {
      return;
    }

    const editorState = mapPortableTaskToEditorState(importedDocument);
    setConfig(editorState.config);
    resetEditorHistory({
      steps: editorState.steps,
      specialPositions: editorState.specialPositions,
      selectedStepId: null,
    });
    setSelectedComponent(null);
  }, [importedDocument, isEditing, resetEditorHistory]);

  useEffect(() => {
    if (!activeRunEnvUuid) {
      setActiveRunEnvStatus(undefined);
      return;
    }

    let cancelled = false;

    const syncEnvironmentStatus = async () => {
      try {
        const status = await getEnvironmentStatus(activeRunEnvUuid);
        if (!cancelled) {
          setActiveRunEnvStatus(status ?? undefined);
        }
      } catch {
        if (!cancelled) {
          setActiveRunEnvStatus(undefined);
        }
      }
    };

    void syncEnvironmentStatus();

    const unlistenPromise = listen<EnvConnectionPayload>(
      ENV_CONNECTION_STATUS_EVENT,
      async (event) => {
        if (event.payload.env_id !== activeRunEnvUuid) {
          return;
        }

        setActiveRunEnvStatus(event.payload.status === 'connected' ? 'running' : 'stopped');
        await syncEnvironmentStatus();
      }
    );

    return () => {
      cancelled = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [activeRunEnvUuid]);

  useEffect(() => {
    if (!activeRunEnvUuid) {
      return;
    }

    if (activeRunEnvStatus === 'running' && runStatus === 'starting') {
      setRunStatus('running');
      return;
    }

    if (activeRunEnvStatus === 'stopped') {
      setActiveRunEnvUuid(null);
      setRunning(false);

      if (runStatus === 'stopping' || stopRequestedRef.current) {
        setRunStatus('stopped');
      } else if (runStatus !== 'success' && runStatus !== 'failed') {
        setRunStatus('stopped');
      }

      stopRequestedRef.current = false;
    }
  }, [activeRunEnvStatus, activeRunEnvUuid, runStatus]);

  const handleSelectComponent = useCallback((component: ComponentItem | null) => {
    setSelectedComponent(component);
  }, []);

  const handleAddComponent = useCallback(
    (
      component: ComponentItem,
      position: { x: number; y: number },
      sourceId?: string | null,
      sourceHandle?: string | null,
      parentLoopId?: string | null
    ) => {
      if (component.type === 'break_loop' && !parentLoopId) {
        toast.warning(
          t('editor.errors.breakLoopOutside', {
            defaultValue: '退出循环节点只能在循环区域内使用。',
          })
        );
        return;
      }

      const newStepId = crypto.randomUUID();
      const branchKey: 'true' | 'false' | null =
        sourceId && sourceHandle ? (sourceHandle === 'branch-false' ? 'false' : 'true') : null;

      const newStep: FlowStep = {
        id: newStepId,
        type: component.type,
        name: component.name,
        config:
          component.type === 'navigate'
            ? { url: DEFAULT_NAVIGATE_URL }
            : component.type === 'select_tab' || component.type === 'close_tab'
              ? { tabIndex: 1 }
            : component.type === 'input'
              ? { text: DEFAULT_INPUT_TEXT }
              : component.type === 'wait'
                ? { waitType: 'time', duration: 1000, timeout: 10000 }
                : component.type === 'condition'
                  ? { conditionType: 'element_visible', expectedText: '', urlFragment: '', branches: {} }
                  : component.type === 'screenshot'
                    ? { captureMode: 'viewport' }
                    : component.type === 'scroll'
                      ? { scrollMode: 'to_element', direction: 'down', viewportCount: 1, edge: 'bottom', block: 'center' }
                      : component.type === 'loop'
                        ? { loopType: 'count', iterations: 3, loopWidth: 380, loopHeight: 280 }
                        : component.type === 'extract'
                          ? {
                              extractType: 'text',
                              outputKey: createDefaultExtractVariableName(),
                              previewValue: '',
                            }
                          : component.type === 'upload'
                            ? {
                                filePath: '',
                              }
                            : component.type === 'script'
                              ? {
                                  script: '',
                                  outputKey: createDefaultScriptVariableName(),
                                  previewValue: '',
                                  executionMode: 'browser',
                                }
                        : {},
        enabled: true,
        isStart: false,
        nextStepId: null,
        position,
        parentLoopId: component.type === 'loop' ? null : parentLoopId ?? null,
      };

      commitEditorChange((current) => {
        let shouldAttachToSource = false;
        if (sourceId === 'start') {
          shouldAttachToSource = true;
        } else if (sourceId) {
          const sourceStep = current.steps.find((step) => step.id === sourceId);
          shouldAttachToSource = Boolean(sourceStep);
        }

        const parentLoop =
          component.type !== 'loop' && parentLoopId
            ? current.steps.find((step) => step.id === parentLoopId && step.type === 'loop') ?? null
            : null;
        const existingLoopChildren = parentLoop
          ? current.steps.filter((step) => step.parentLoopId === parentLoop.id && step.id !== newStepId).length
          : 0;
        const shouldUseAutoLayout = parentLoop && component.type !== 'loop' && !sourceId;
        const loopAwareStep =
          parentLoop && component.type !== 'loop'
            ? {
                ...newStep,
                isStart: sourceId === 'start' && shouldAttachToSource,
                position: shouldUseAutoLayout ? getLoopAutoPosition(parentLoop, existingLoopChildren) : position,
                parentLoopId: parentLoop.id,
              }
            : {
                ...newStep,
                isStart: sourceId === 'start' && shouldAttachToSource,
              };

        const nextSteps = current.steps.map((step) => {
          if (sourceId === 'start') {
            if (shouldAttachToSource) {
              return { ...step, isStart: false };
            }
            return step;
          }

          if (parentLoop && step.id === parentLoop.id) {
            const dimensions = getLoopAutoDimensions(step, existingLoopChildren + 1);
            return {
              ...step,
              config: {
                ...step.config,
                loopWidth: dimensions.width,
                loopHeight: dimensions.height,
              },
            };
          }

          if (sourceId && step.id === sourceId && shouldAttachToSource) {
            if (step.type === 'condition' && branchKey) {
              const currentBranches = (step.config.branches as Record<string, unknown> | undefined) ?? {};
              return {
                ...step,
                config: {
                  ...step.config,
                  branches: {
                    ...currentBranches,
                    [branchKey]: newStepId,
                  },
                },
              };
            }

            return { ...step, nextStepId: newStepId };
          }

          return step;
        });

        return {
          ...current,
          steps: [...nextSteps, loopAwareStep],
          selectedStepId: newStep.id,
        };
      });
      setSelectedComponent(null);
    },
    [commitEditorChange, t]
  );

  const handleUpdateStep = useCallback((updatedStep: FlowStep) => {
    commitEditorChange((current) => ({
      ...current,
      steps: current.steps.map((step) => (step.id === updatedStep.id ? updatedStep : step)),
      selectedStepId: updatedStep.id,
    }));
  }, [commitEditorChange]);

  const handleGlobalVariablesChange = useCallback((variables: TaskVariable[]) => {
    setConfig((prev) => ({ ...prev, globalVariables: variables }));
  }, []);

  const handleDeleteStep = useCallback(
    (stepId: string) => {
      commitEditorChange((current) => {
        const deletedStep = current.steps.find((step) => step.id === stepId);
        const fallbackStartId = deletedStep?.isStart ? deletedStep.nextStepId ?? null : null;
        const deletedLoopPosition = deletedStep?.type === 'loop' ? deletedStep.position ?? { x: 0, y: 0 } : null;
        const deletedStepIds = new Set(
          current.steps
            .filter(
              (step) =>
                step.id === stepId ||
                (deletedStep?.type === 'loop' &&
                  step.parentLoopId === stepId &&
                  step.type === 'break_loop')
            )
            .map((step) => step.id)
        );

        const nextSteps = current.steps
          .filter((step) => !deletedStepIds.has(step.id))
          .map((step) => {
            const nextStepId =
              step.nextStepId && deletedStepIds.has(step.nextStepId) ? null : step.nextStepId ?? null;
            const currentBranches = (step.config.branches as Record<string, unknown> | undefined) ?? {};
            const branches = Object.fromEntries(
              Object.entries(currentBranches).map(([key, value]) => [
                key,
                typeof value === 'string' && deletedStepIds.has(value) ? null : value,
              ])
            );
            const isStart = fallbackStartId ? step.id === fallbackStartId : step.isStart ?? false;
            const wasInsideDeletedLoop = step.parentLoopId === stepId && deletedLoopPosition;

            return {
              ...step,
              nextStepId,
              isStart,
              parentLoopId: wasInsideDeletedLoop ? null : step.parentLoopId ?? null,
              position:
                wasInsideDeletedLoop && step.position
                  ? {
                      x: deletedLoopPosition.x + step.position.x,
                      y: deletedLoopPosition.y + step.position.y,
                    }
                  : step.position,
              config:
                step.type === 'condition'
                  ? {
                      ...step.config,
                      branches,
                    }
                  : step.config,
            };
          });

        return {
          ...current,
          steps: nextSteps,
          selectedStepId: current.selectedStepId === stepId ? null : current.selectedStepId,
        };
      });
    },
    [commitEditorChange]
  );

  const handleUpdateSteps = useCallback((newSteps: FlowStep[]) => {
    commitEditorChange((current) => ({
      ...current,
      steps: newSteps,
      selectedStepId:
        current.selectedStepId && newSteps.some((step) => step.id === current.selectedStepId)
          ? current.selectedStepId
          : null,
    }));
  }, [commitEditorChange]);

  const handleSpecialPositionsChange = useCallback((positions: SpecialNodePositions) => {
    commitEditorChange((current) => ({
      ...current,
      specialPositions: positions,
    }));
  }, [commitEditorChange]);

  const handleSave = async () => {
    if (!config.name.trim()) {
      toast.warning(t('editor.errors.nameRequired'));
      return;
    }

    if (steps.length === 0) {
      toast.warning(t('editor.errors.stepsRequired'));
      return;
    }

    if (findInvalidBreakLoopStep(steps)) {
      toast.warning(
        t('editor.errors.breakLoopOutside', {
          defaultValue: '退出循环节点只能在循环区域内使用。',
        })
      );
      return;
    }

    setSaving(true);
    try {
      const payload = buildTaskPayload(config, steps, specialPositions);
      if (isEditing && id) {
        await updateRpaTask({ ...payload, uuid: id });
      } else {
        await createRpaTask(payload);
      }

      navigate('/rpa');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : 'Failed to save task.');
    } finally {
      setSaving(false);
    }
  };

  const handleStepStatusChange = useCallback((event: RpaRunnerStepEvent) => {
    setRunStepItems((prev) =>
      prev.map((step) =>
        step.id === event.stepId
          ? {
              ...step,
              status: event.status,
              error: normalizeRunError(event.error),
              loopIteration: event.loopIteration ?? step.loopIteration,
              loopTotal: event.loopTotal ?? step.loopTotal,
              incomingFromStepId: event.incomingFromStepId ?? step.incomingFromStepId,
              incomingBranchKey: event.incomingBranchKey ?? step.incomingBranchKey,
            }
          : step
      )
    );
  }, [normalizeRunError]);

  const handleTargetCaptured = useCallback((event: RpaRunnerCapturedTargetEvent) => {
    setSteps((prev) =>
      prev.map((step) =>
        step.id === event.stepId
          ? {
              ...step,
              config: {
                ...step.config,
                selector: event.captured.primary.value,
                selectorCandidates: event.captured.candidates,
                selectorSnapshot: event.captured.snapshot,
              },
            }
          : step
      )
    );
  }, []);

  const handleDataExtracted = useCallback((event: RpaRunnerExtractedDataEvent) => {
    setSteps((prev) =>
      prev.map((step) =>
        step.id === event.stepId
          ? {
              ...step,
              config: {
                ...step.config,
                previewValue: event.value,
              },
            }
          : step
      )
    );
  }, []);
  const handleScreenshotCaptured = useCallback((event: RpaRunnerScreenshotEvent) => {
    setSteps((prev) =>
      prev.map((step) =>
        step.id === event.stepId
          ? {
              ...step,
              config: {
                ...step.config,
                lastScreenshot: `data:image/png;base64,${event.screenshot}`,
              },
            }
          : step
      )
    );
  }, []);

  const executeWorkflow = useCallback(
    async (workflow: RpaWorkflowSchema) => {
      setRunStepItems(
        workflow.steps.map((step) => ({
          id: step.id,
          name: step.name,
          type: step.type,
          status: 'pending',
          error: undefined,
          loopIteration: undefined,
          loopTotal:
            step.type === 'loop' &&
            typeof step.config?.iterations === 'number' &&
            Number.isFinite(step.config.iterations)
              ? Math.max(1, Math.trunc(step.config.iterations))
              : undefined,
          incomingFromStepId: undefined,
          incomingBranchKey: undefined,
        }))
      );
      setActiveRunEnvUuid(null);
      setActiveRunEnvStatus('starting');
      setRunStatus('starting');
      setRunning(true);
      stopRequestedRef.current = false;

      try {
        const endpoint = await startAnonymousRpaEnvironment(anonymousProxyConfig);
        setActiveRunEnvUuid(endpoint.env_uuid);

        const runner = new RpaRunner(new CdpBrowserAdapter());
        await runner.run(
          workflow,
          {
            envUuid: endpoint.env_uuid,
            browserWsUrl: endpoint.browser_ws_url || '',
          },
          {
            onStepStatusChange: handleStepStatusChange,
            onTargetCaptured: handleTargetCaptured,
            onDataExtracted: handleDataExtracted,
            onScreenshotCaptured: handleScreenshotCaptured,
          }
        );

        setRunStatus('success');
      } catch {
        if (stopRequestedRef.current) {
          setRunStatus('stopped');
        } else {
          setRunStatus('failed');
        }
      } finally {
        setRunning(false);
      }
    },
    [
      anonymousProxyConfig,
      handleDataExtracted,
      handleScreenshotCaptured,
      handleStepStatusChange,
      handleTargetCaptured,
    ]
  );

  const handleRun = async () => {
    if (runButtonBusy) {
      return;
    }

    if (steps.length === 0) {
      toast.warning(t('editor.errors.stepsRequired'));
      return;
    }

    if (findInvalidBreakLoopStep(steps)) {
      toast.warning(
        t('editor.errors.breakLoopOutside', {
          defaultValue: '退出循环节点只能在循环区域内使用。',
        })
      );
      return;
    }

    const workflow = buildWorkflowSchema(config, steps);
    if (!workflow.start_step_id) {
      toast.warning(t('editor.errors.startPathRequired'));
      return;
    }

    await executeWorkflow(workflow);
  };

  const handleStopRunningEnvironment = async () => {
    if (!activeRunEnvUuid || runStatus === 'stopping' || runStatus === 'stopped') {
      return;
    }

    stopRequestedRef.current = true;
    setActiveRunEnvStatus('stopping');
    setRunStatus('stopping');

    try {
      await stopAnonymousRpaEnvironment(activeRunEnvUuid);
    } catch {
      stopRequestedRef.current = false;
      setRunStatus('failed');
    }
  };

  const handleBack = () => {
    if (steps.length > 0 || config.name) {
      if (!window.confirm(t('editor.confirmLeave'))) {
        return;
      }
    }
    navigate('/rpa');
  };

  const handleClearSteps = () => {
    if (steps.length === 0) {
      return;
    }

    if (!window.confirm(t('editor.menu.clearSteps'))) {
      return;
    }

    commitEditorChange(() => ({
      steps: [],
      specialPositions: DEFAULT_SPECIAL_NODE_POSITIONS,
      selectedStepId: null,
    }));
    setSelectedComponent(null);
    setRunStepItems([]);
    setRunStatus('idle');
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.shiftKey || event.key.toLowerCase() !== 'z') {
        return;
      }

      if (isEditableUndoTarget(event.target) || !canUndo) {
        return;
      }

      event.preventDefault();
      handleUndo();
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [canUndo, handleUndo]);

  return (
    <div className="flex flex-col h-[calc(100vh-50px)] relative">
      <div className="h-12 border-b border-border bg-background flex items-center justify-between px-4">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={handleBack}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <Input
            value={config.name}
            onChange={(e) => setConfig((prev) => ({ ...prev, name: e.target.value }))}
            placeholder={t('editor.taskNamePlaceholder')}
            className="w-64 h-8 text-sm font-medium border-0 bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0 px-0 shadow-none"
          />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setSettingsOpen(true)}
            className="flex items-center gap-1.5 text-xs font-semibold px-3 py-2 border border-border rounded hover:bg-accent transition-colors"
          >
            <Settings className="h-3.5 w-3.5" />
            {t('editor.settings')}
          </button>
          <div className="inline-flex overflow-hidden rounded border border-border">
            <button
              onClick={runButtonBusy ? handleStopRunningEnvironment : handleRun}
              disabled={runStatus === 'stopping'}
              className="flex items-center gap-1.5 text-xs font-semibold px-3 py-2 hover:bg-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {runButtonBusy ? (
                <Square className="h-3.5 w-3.5" />
              ) : (
                <Play className="h-3.5 w-3.5" />
              )}
              {runButtonBusy ? t('table.actions.stop') : t('editor.run')}
            </button>
            <button
              type="button"
              onClick={(event) => {
                event.stopPropagation();
                setProxyDrawerOpen(true);
              }}
              disabled={proxyButtonDisabled}
              title={anonymousProxyLabel}
              aria-label={t('editor.proxy.openDrawer', { defaultValue: '配置匿名环境代理' })}
              className={cn(
                'relative flex items-center justify-center px-2.5 py-2 border-l border-border hover:bg-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed',
                selectedAnonymousProxy ? 'text-primary' : 'text-muted-foreground'
              )}
            >
              <Shield className="h-3.5 w-3.5" />
              {selectedAnonymousProxy ? (
                <span className="absolute right-1.5 top-1.5 h-1.5 w-1.5 rounded-full bg-primary" />
              ) : null}
            </button>
          </div>
          <button
            onClick={handleSave}
            disabled={saving}
            className="flex items-center gap-1.5 bg-primary text-primary-foreground text-xs font-semibold px-4 py-2 hover:bg-primary/90 transition-colors border border-primary rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Save className="h-3.5 w-3.5" />
            {saving ? t('editor.saving') : t('editor.save')}
          </button>
        </div>
      </div>

      <AnonymousProxyDrawer
        open={proxyDrawerOpen}
        value={selectedAnonymousProxy}
        onOpenChange={setProxyDrawerOpen}
        onChange={setSelectedAnonymousProxy}
      />

      <div className="flex-1 flex min-h-0">
        <ComponentPanel
          selectedComponent={selectedComponent}
          onSelectComponent={handleSelectComponent}
        />
        <FlowCanvas
          steps={steps}
          specialPositions={specialPositions}
          selectedStepId={activeStepId ?? selectedStepId}
          onSelectStep={setSelectedStepId}
          onDeleteStep={handleDeleteStep}
          onUpdateSteps={handleUpdateSteps}
          onSpecialPositionsChange={handleSpecialPositionsChange}
          onDropComponent={handleAddComponent}
          onUndo={handleUndo}
          onClearSteps={handleClearSteps}
          undoDisabled={!canUndo}
          clearStepsDisabled={runButtonBusy || steps.length === 0}
          draggingComponent={selectedComponent}
          runStatus={runStatus}
          stepStatuses={stepStatusMap}
          stepErrors={stepErrorMap}
          stepLoopProgress={loopProgressMap}
          stepIncomingEdges={stepIncomingEdgeMap}
        />
        <div className="w-80 border-l border-border bg-background flex flex-col min-h-0 overflow-hidden">
          <Accordion
            type="single"
            collapsible
            value={sidebarSection}
            onValueChange={(value) => setSidebarSection((value as 'properties' | 'variables' | '') ?? '')}
            className="flex-1 min-h-0 flex flex-col"
          >
            <AccordionItem
              value="properties"
              className={
                sidebarSection === 'properties'
                  ? 'flex flex-1 min-h-0 flex-col [&>[data-slot=accordion-content]]:flex-1 [&>[data-slot=accordion-content]]:min-h-0'
                  : 'shrink-0'
              }
            >
              <AccordionTrigger className="px-3 py-2 text-xs font-semibold hover:no-underline">{t('editor.properties')}</AccordionTrigger>
              <AccordionContent
                className={
                  sidebarSection === 'properties'
                    ? 'flex h-full flex-1 min-h-0 flex-col overflow-hidden pb-0'
                    : 'pb-0'
                }
              >
                <div className="flex flex-1 min-h-0 flex-col overflow-hidden">
                  <PropertyPanel
                    embedded
                    step={selectedStep}
                    onUpdateStep={handleUpdateStep}
                    onDeleteStep={handleDeleteStep}
                  />
                </div>
              </AccordionContent>
            </AccordionItem>
            <AccordionItem
              value="variables"
              className={
                sidebarSection === 'variables'
                  ? 'flex flex-1 min-h-0 flex-col [&>[data-slot=accordion-content]]:flex-1 [&>[data-slot=accordion-content]]:min-h-0'
                  : 'shrink-0'
              }
            >
              <AccordionTrigger className="px-3 py-2 text-xs font-semibold hover:no-underline">
                {t('editor.variablePanel.title', { defaultValue: '变量' })}
              </AccordionTrigger>
              <AccordionContent
                className={
                  sidebarSection === 'variables'
                    ? 'flex h-full flex-1 min-h-0 flex-col overflow-hidden pb-0'
                    : 'pb-0'
                }
              >
                <div className="flex flex-1 min-h-0 flex-col overflow-hidden">
                  <VariablePanel
                    embedded
                    globalVariables={config.globalVariables}
                    runtimeVariables={runtimeVariables}
                    onGlobalVariablesChange={handleGlobalVariablesChange}
                  />
                </div>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </div>
      </div>

      {loadingTask && (
        <div className="absolute inset-0 bg-background/70 backdrop-blur-[1px] flex items-center justify-center text-sm text-muted-foreground">
          {t('loading')}
        </div>
      )}

      <FormattedDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        header={{
          icon: Settings2,
          title: t('editor.taskSettings'),
          description: t('editor.taskSettingsDescription', {
            defaultValue: '配置任务的基础信息、触发方式、环境和执行策略。',
          }),
        }}
        contentClassName="h-[60vh]"
        contentPadding="p-0"
        minWidth="min-w-[620px]"
      >
        <div className="h-full px-5 py-4">
          <TaskSettingsForm config={config} onConfigChange={setConfig} />
        </div>
        <FormattedDialogFooter>
          <Button
            variant="outline"
            size="sm"
            className="text-xs"
            onClick={() => setSettingsOpen(false)}
          >
            {t('dialog.settings.cancel')}
          </Button>
          <Button
            size="sm"
            className="text-xs"
            onClick={() => setSettingsOpen(false)}
          >
            {t('dialog.settings.confirm')}
          </Button>
        </FormattedDialogFooter>
      </FormattedDialog>
    </div>
  );
}

export default TaskEditor;










