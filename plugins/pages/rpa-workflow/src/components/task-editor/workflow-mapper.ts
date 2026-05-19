import type { RpaTaskDetailDto, UpsertRpaTaskRequest } from '../../api';
import type { PortableRpaTaskDocument } from '../../lib/rpa-transfer';
import type { RpaWorkflowSchema, RpaWorkflowStep } from '../../types';
import {
  DEFAULT_SPECIAL_NODE_POSITIONS,
  type FlowStep,
  type SpecialNodePositions,
} from './flow-canvas';
import type { TaskConfig, TaskVariable } from './task-settings-form';

const DEFAULT_NAVIGATE_URL = 'https://www.google.com';
const DEFAULT_INPUT_TEXT = 'Hello World';

interface EditorState {
  config: TaskConfig;
  steps: FlowStep[];
  specialPositions: SpecialNodePositions;
  workflow: RpaWorkflowSchema;
}

interface StoredWorkflowStep {
  id?: string;
  type?: string;
  name?: string;
  enabled?: boolean;
  next?: string | null;
  branches?: Record<string, string>;
  config?: Record<string, unknown>;
  ui?: {
    source_type?: string;
    position?: {
      x?: number;
      y?: number;
    };
    parent_id?: string | null;
  };
}

interface StoredWorkflowMeta {
  start_step_id?: string | null;
  global_variables?: Array<{
    name?: string;
    value?: string;
    prompt_on_run?: boolean;
    required?: boolean;
  }>;
  start_position?: {
    x?: number;
    y?: number;
  };
  end_position?: {
    x?: number;
    y?: number;
  };
}

function buildSelector(selector: unknown) {
  if (typeof selector !== 'string' || !selector.trim()) {
    return undefined;
  }

  const value = selector.trim();
  const by = value.startsWith('/') || value.startsWith('(') ? 'xpath' : 'css';

  return {
    by,
    value,
  };
}

function asNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function readCanvasPosition(
  value: unknown,
  fallback: { x: number; y: number }
): { x: number; y: number } {
  if (
    value &&
    typeof value === 'object' &&
    typeof (value as { x?: unknown }).x === 'number' &&
    typeof (value as { y?: unknown }).y === 'number'
  ) {
    return {
      x: (value as { x: number }).x,
      y: (value as { y: number }).y,
    };
  }

  return fallback;
}

function mapFlowStepType(type: string, config: Record<string, unknown>): string {
  switch (type) {
    case 'navigate':
      return 'goto';
    case 'input':
      return 'fill';
    case 'wait':
      return (config.waitType as string) === 'time' ? 'delay' : 'wait_for';
    case 'extract':
      return 'extract_text';
    case 'script':
      return 'execute_js';
    case 'variable':
      return 'set_variable';
    default:
      return type;
  }
}

function buildWorkflowStep(step: FlowStep, nextStepId: string | null): RpaWorkflowStep {
  const type = mapFlowStepType(step.type, step.config);
  const baseStep: RpaWorkflowStep = {
    id: step.id,
    type,
    name: step.name,
    enabled: step.enabled,
    next: nextStepId,
    ui: {
      source_type: step.type,
      position: step.position ?? { x: 0, y: 0 },
      parent_id: step.parentLoopId ?? null,
    },
    config: { ...step.config },
  };

  switch (type) {
    case 'goto':
      return {
        ...baseStep,
        url:
          typeof step.config.url === 'string' && step.config.url.trim()
            ? step.config.url.trim()
            : DEFAULT_NAVIGATE_URL,
        wait_until: step.config.waitForLoad === false ? 'commit' : 'load',
      };
    case 'select_tab':
    case 'close_tab':
      return {
        ...baseStep,
        config: {
          ...baseStep.config,
          tabIndex: asNumber(step.config.tabIndex, 1),
        },
      };
    case 'click':
      return {
        ...baseStep,
        target: buildSelector(step.config.selector),
        click_type: (step.config.clickType as string) || 'single',
        wait_for_element: step.config.waitForElement !== false,
      };
    case 'fill':
      return {
        ...baseStep,
        target: buildSelector(step.config.selector),
        value:
          typeof step.config.text === 'string' && step.config.text.trim()
            ? step.config.text
            : DEFAULT_INPUT_TEXT,
        clear_first: step.config.clearFirst !== false,
        type_slowly: step.config.typeSlowly === true,
      };
    case 'delay':
      return {
        ...baseStep,
        duration_ms: asNumber(step.config.duration, 1000),
      };
    case 'wait_for':
      return {
        ...baseStep,
        target: buildSelector(step.config.selector),
        wait_type: 'element',
        timeout_ms: asNumber(step.config.timeout, 10000),
      };
    case 'upload':
      return {
        ...baseStep,
        target: buildSelector(step.config.selector),
        config: {
          ...baseStep.config,
          filePath: typeof step.config.filePath === 'string' ? step.config.filePath : '',
        },
      };

    case 'scroll': {
      const scrollMode =
        step.config.scrollMode === 'by_viewport'
          ? 'by_viewport'
          : step.config.scrollMode === 'to_edge'
            ? 'to_edge'
            : 'to_element';

      return {
        ...baseStep,
        target: scrollMode === 'to_element' ? buildSelector(step.config.selector) : undefined,
        config: {
          ...baseStep.config,
          scrollMode,
          direction: step.config.direction === 'up' ? 'up' : 'down',
          viewportCount: asNumber(step.config.viewportCount, 1),
          edge: step.config.edge === 'top' ? 'top' : 'bottom',
          block:
            step.config.block === 'start' ||
            step.config.block === 'end' ||
            step.config.block === 'nearest'
              ? step.config.block
              : 'center',
        },
      };
    }

    case 'extract_text':
      return {
        ...baseStep,
        target: buildSelector(step.config.selector),
        output: typeof step.config.outputKey === 'string' ? step.config.outputKey : undefined,
        config: {
          ...baseStep.config,
          extractType:
            step.config.extractType === 'href'
              ? 'href'
              : step.config.extractType === 'value'
                ? 'value'
                : 'text',
          outputKey: typeof step.config.outputKey === 'string' ? step.config.outputKey : '',
          previewValue: typeof step.config.previewValue === 'string' ? step.config.previewValue : '',
        },
      };
    case 'execute_js':
      return {
        ...baseStep,
        script: typeof step.config.script === 'string' ? step.config.script : '',
        output: typeof step.config.outputKey === 'string' ? step.config.outputKey : undefined,
        config: {
          ...baseStep.config,
          outputKey: typeof step.config.outputKey === 'string' ? step.config.outputKey : '',
          previewValue: typeof step.config.previewValue === 'string' ? step.config.previewValue : '',
        },
      };
    case 'condition': {
      const conditionType =
        step.config.conditionType === 'text_present'
          ? 'text_present'
          : step.config.conditionType === 'url_contains'
            ? 'url_contains'
            : 'element_visible';

      return {
        ...baseStep,
        target: conditionType === 'element_visible' ? buildSelector(step.config.selector) : undefined,
        branches:
          step.config.branches && typeof step.config.branches === 'object'
            ? (step.config.branches as Record<string, string>)
            : undefined,
        config: {
          ...baseStep.config,
          conditionType,
          expectedText:
            typeof step.config.expectedText === 'string' ? step.config.expectedText : '',
          urlFragment:
            typeof step.config.urlFragment === 'string' ? step.config.urlFragment : '',
        },
      };
    }
    case 'loop': {
      return {
        ...baseStep,
        config: {
          ...baseStep.config,
          loopType: step.config.loopType === 'count' ? 'count' : 'count',
          iterations: asNumber(step.config.iterations, 3),
        },
      };
    }
    case 'break_loop':
      return {
        ...baseStep,
        next: null,
      };
    case 'screenshot': {
      const screenshotMode =
        step.config.captureMode === 'element'
          ? 'element'
          : step.config.captureMode === 'full_page'
            ? 'full_page'
            : 'viewport';

      return {
        ...baseStep,
        target: screenshotMode === 'element' ? buildSelector(step.config.selector) : undefined,
        screenshot_mode: screenshotMode,
        config: {
          ...baseStep.config,
          captureMode: screenshotMode,
        },
      };
    }
    default:
      return baseStep;
  }
}

function toStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.map(String).filter(Boolean);
}

function mapStoredGlobalVariables(
  variables: StoredWorkflowMeta['global_variables'] | undefined
): TaskVariable[] {
  if (!Array.isArray(variables)) {
    return [];
  }

  return variables
    .map((variable, index) => ({
      id: `var-${index}`,
      name: typeof variable?.name === 'string' ? variable.name : '',
      value: typeof variable?.value === 'string' ? variable.value : '',
      promptOnRun: variable?.prompt_on_run === true,
      required: variable?.required === true,
    }))
    .filter((variable) => variable.name.trim());
}

function buildFlowStepFromStoredStep(step: StoredWorkflowStep): FlowStep {
  const uiType = typeof step.ui?.source_type === 'string' ? step.ui.source_type : step.type;
  const position = step.ui?.position;
  const parentLoopId = typeof step.ui?.parent_id === 'string' ? step.ui.parent_id : null;
  const config =
    step.config && typeof step.config === 'object'
      ? { ...(step.config as Record<string, unknown>) }
      : {};

  if (step.branches && typeof step.branches === 'object') {
    config.branches = step.branches;
  }

  return {
    id: String(step.id),
    type: uiType,
    name: typeof step.name === 'string' ? step.name : String(step.type),
    config,
    enabled: step.enabled !== false,
    nextStepId: typeof step.next === 'string' ? step.next : null,
    position:
      position && typeof position.x === 'number' && typeof position.y === 'number'
        ? { x: position.x, y: position.y }
        : undefined,
    parentLoopId,
  };
}

function buildLegacyFlowStep(
  step: RpaTaskDetailDto['steps'][number],
  nextStepId: string | null
): FlowStep {
  return {
    id: step.uuid,
    type: step.step_type,
    name: step.name,
    config: {
      ...(step.config || {}),
      ...(step.branch_config && typeof step.branch_config === 'object'
        ? { branches: step.branch_config }
        : {}),
    },
    enabled: step.enabled ?? true,
    nextStepId,
    position:
      typeof step.position_x === 'number' && typeof step.position_y === 'number'
        ? { x: step.position_x, y: step.position_y }
        : undefined,
  };
}

export function buildWorkflowSchema(config: TaskConfig, steps: FlowStep[]): RpaWorkflowSchema {
  const workflowSteps = steps.map((step) => buildWorkflowStep(step, step.nextStepId ?? null));

  return {
    schema_version: '1.0',
    name: config.name.trim(),
    description: config.description,
    tags: config.tags,
    trigger: {
      type: config.triggerType,
      schedule: config.schedule,
      cron_expression: config.cronExpression,
    },
    environments: config.selectedEnvironments,
    settings: {
      run_mode: config.runMode,
      retry_count: config.retryCount,
      retry_interval_seconds: config.retryInterval,
      timeout_seconds: config.timeout,
      concurrency: config.concurrency,
      stop_on_error: config.stopOnError,
      notify_on_complete: config.notifyOnComplete,
      notify_on_error: config.notifyOnError,
    },
    variables: config.globalVariables
      .map((variable) => ({ name: variable.name.trim(), value: variable.value }))
      .filter((variable) => variable.name),
    start_step_id: steps.find((step) => step.isStart)?.id ?? null,
    steps: workflowSteps,
  };
}

export function buildTaskPayload(
  config: TaskConfig,
  steps: FlowStep[],
  specialPositions: SpecialNodePositions
): UpsertRpaTaskRequest {
  const workflow = buildWorkflowSchema(config, steps);

  return {
    name: workflow.name,
    description: workflow.description || undefined,
    tags: workflow.tags.length ? workflow.tags : undefined,
    trigger_type: workflow.trigger.type,
    schedule: workflow.trigger.schedule || undefined,
    cron_expression: workflow.trigger.cron_expression || undefined,
    run_mode: workflow.settings.run_mode,
    retry_count: workflow.settings.retry_count,
    retry_interval: workflow.settings.retry_interval_seconds,
    timeout: workflow.settings.timeout_seconds,
    concurrency: workflow.settings.concurrency,
    stop_on_error: workflow.settings.stop_on_error,
    notify_on_complete: workflow.settings.notify_on_complete,
    notify_on_error: workflow.settings.notify_on_error,
    environment_uuids: workflow.environments.length ? workflow.environments : undefined,
    steps: workflow.steps.map((step, index) => ({
      step_type: step.type,
      name: step.name,
      config: {
        workflow_step: step,
        workflow_meta: {
          start_step_id: workflow.start_step_id,
          global_variables: config.globalVariables
            .map((variable) => ({
              name: variable.name.trim(),
              value: variable.value,
              prompt_on_run: variable.promptOnRun === true,
              required: variable.promptOnRun === true && variable.required === true,
            }))
            .filter((variable) => variable.name),
          start_position: specialPositions.start,
          end_position: specialPositions.end,
        },
        legacy_config: steps[index]?.config ?? {},
      },
      enabled: step.enabled,
      position_x:
        typeof step.ui?.position?.x === 'number' ? Math.round(step.ui.position.x) : undefined,
      position_y:
        typeof step.ui?.position?.y === 'number' ? Math.round(step.ui.position.y) : undefined,
      sort_order: index,
      next_step_uuid: step.next,
      branch_config: step.branches,
    })),
  };
}

export function mapTaskDetailToEditorState(detail: RpaTaskDetailDto): EditorState {
  const sortedSteps = [...detail.steps].sort(
    (a, b) => (a.sort_order ?? Number.MAX_SAFE_INTEGER) - (b.sort_order ?? Number.MAX_SAFE_INTEGER)
  );

  const storedWorkflowMeta = sortedSteps.find((step) => step.config?.workflow_meta)?.config
    ?.workflow_meta as StoredWorkflowMeta | undefined;
  const storedStartStepId =
    typeof storedWorkflowMeta?.start_step_id === 'string' ? storedWorkflowMeta.start_step_id : null;
  const storedGlobalVariables = mapStoredGlobalVariables(storedWorkflowMeta?.global_variables);
  const specialPositions: SpecialNodePositions = {
    start: readCanvasPosition(
      storedWorkflowMeta?.start_position,
      DEFAULT_SPECIAL_NODE_POSITIONS.start
    ),
    end: readCanvasPosition(
      storedWorkflowMeta?.end_position,
      DEFAULT_SPECIAL_NODE_POSITIONS.end
    ),
  };

  const steps = sortedSteps.map((step, index) => {
    const storedStep = step.config?.workflow_step;
    if (storedStep && typeof storedStep === 'object') {
      return buildFlowStepFromStoredStep(storedStep as StoredWorkflowStep);
    }

    return buildLegacyFlowStep(step, sortedSteps[index + 1]?.uuid ?? null);
  }).map((step, index) => ({
    ...step,
    isStart: storedStartStepId ? step.id === storedStartStepId : index === 0,
  }));

  const config: TaskConfig = {
    name: detail.task.name,
    description: detail.task.description ?? '',
    tags: toStringArray(detail.task.tags),
    triggerType:
      detail.task.trigger_type === 'scheduled' || detail.task.trigger_type === 'event'
        ? detail.task.trigger_type
        : 'manual',
    schedule: detail.task.schedule ?? undefined,
    cronExpression: detail.task.cron_expression ?? undefined,
    selectedEnvironments: detail.environment_uuids || [],
    runMode: detail.task.run_mode === 'parallel' ? 'parallel' : 'sequential',
    globalVariables: storedGlobalVariables,
    retryCount: detail.task.retry_count ?? 3,
    retryInterval: detail.task.retry_interval ?? 5,
    timeout: detail.task.timeout ?? 300,
    concurrency: detail.task.concurrency ?? 1,
    stopOnError: detail.task.stop_on_error ?? true,
    notifyOnComplete: detail.task.notify_on_complete ?? false,
    notifyOnError: detail.task.notify_on_error ?? true,
  };

  return {
    config,
    steps,
    specialPositions,
    workflow: buildWorkflowSchema(config, steps),
  };
}





export function mapPortableTaskToEditorState(document: PortableRpaTaskDocument): EditorState {
  const sortedSteps = [...document.task.steps].sort(
    (a, b) => (a.sort_order ?? Number.MAX_SAFE_INTEGER) - (b.sort_order ?? Number.MAX_SAFE_INTEGER)
  );

  const storedWorkflowMeta = sortedSteps.find((step) => {
    const config = step.config;
    return config && typeof config === 'object' && 'workflow_meta' in config;
  })?.config?.workflow_meta as StoredWorkflowMeta | undefined;
  const importedIdMap = new Map<string, string>();
  sortedSteps.forEach((step) => {
    importedIdMap.set(step.uuid, crypto.randomUUID());
  });
  const storedStartStepId =
    typeof storedWorkflowMeta?.start_step_id === 'string'
      ? importedIdMap.get(storedWorkflowMeta.start_step_id) ?? null
      : null;

  const steps = sortedSteps.map((step, index) => {
    const legacyStep = buildLegacyFlowStep(
      {
        uuid: step.uuid,
        step_type: step.step_type,
        name: step.name,
        config: step.config,
        enabled: step.enabled ?? true,
        position_x: step.position_x,
        position_y: step.position_y,
        sort_order: step.sort_order,
        next_step_uuid: step.next_step_uuid,
        branch_config: step.branch_config,
      },
      step.next_step_uuid ?? sortedSteps[index + 1]?.uuid ?? null
    );

    const currentBranches = (legacyStep.config.branches as Record<string, unknown> | undefined) ?? {};
    const mappedBranches = Object.fromEntries(
      Object.entries(currentBranches).map(([key, value]) => [
        key,
        typeof value === 'string' ? importedIdMap.get(value) ?? null : value,
      ])
    );

    return {
      ...legacyStep,
      id: importedIdMap.get(step.uuid) ?? legacyStep.id,
      nextStepId: step.next_step_uuid ? importedIdMap.get(step.next_step_uuid) ?? null : null,
      isStart: storedStartStepId ? importedIdMap.get(step.uuid) === storedStartStepId : index === 0,
      config:
        Object.keys(mappedBranches).length > 0
          ? {
              ...legacyStep.config,
              branches: mappedBranches,
            }
          : legacyStep.config,
    };
  });

  const config: TaskConfig = {
    name: document.task.name,
    description: document.task.description ?? '',
    tags: toStringArray(document.task.tags),
    triggerType:
      document.task.trigger_type === 'scheduled' || document.task.trigger_type === 'event'
        ? document.task.trigger_type
        : 'manual',
    schedule: document.task.schedule ?? undefined,
    cronExpression: document.task.cron_expression ?? undefined,
    selectedEnvironments: document.task.environment_uuids || [],
    runMode: document.task.run_mode === 'parallel' ? 'parallel' : 'sequential',
    globalVariables: mapStoredGlobalVariables(storedWorkflowMeta?.global_variables),
    retryCount: document.task.retry_count ?? 3,
    retryInterval: document.task.retry_interval ?? 5,
    timeout: document.task.timeout ?? 300,
    concurrency: document.task.concurrency ?? 1,
    stopOnError: document.task.stop_on_error ?? true,
    notifyOnComplete: document.task.notify_on_complete ?? false,
    notifyOnError: document.task.notify_on_error ?? true,
  };

  return {
    config,
    steps,
    specialPositions: {
      start: readCanvasPosition(
        storedWorkflowMeta?.start_position,
        DEFAULT_SPECIAL_NODE_POSITIONS.start
      ),
      end: readCanvasPosition(
        storedWorkflowMeta?.end_position,
        DEFAULT_SPECIAL_NODE_POSITIONS.end
      ),
    },
    workflow: buildWorkflowSchema(config, steps),
  };
}
