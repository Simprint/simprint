import { isSuccess, post } from '@/lib/request';
import type { RpaTask, RpaWorkflowSchema } from '../types';

export const API_ENDPOINTS = {
  LIST_TASKS: 'rpa/tasks',
  GET_TASK_DETAIL: 'rpa/tasks/detail',
  CREATE_TASK: 'rpa/tasks/create',
  UPDATE_TASK: 'rpa/tasks/update',
  DELETE_TASK: 'rpa/tasks/delete',
  BATCH_DELETE_TASKS: 'rpa/tasks/batch-delete',
  RUN_TASK: 'rpa/tasks/run',
  STOP_TASK: 'rpa/tasks/stop',
  DUPLICATE_TASK: 'rpa/tasks/duplicate',
  LIST_ENVIRONMENTS: 'environments/list',
} as const;

export interface RpaTaskDto {
  id: number;
  uuid: string;
  name: string;
  description?: string | null;
  tags?: unknown;
  trigger_type: string;
  schedule?: string | null;
  cron_expression?: string | null;
  run_mode: string;
  retry_count?: number | null;
  retry_interval?: number | null;
  timeout?: number | null;
  concurrency?: number | null;
  stop_on_error?: boolean | null;
  notify_on_complete?: boolean | null;
  notify_on_error?: boolean | null;
  status: string;
  run_count?: number | null;
  success_count?: number | null;
  environment_count?: number | null;
  last_run_at?: string | null;
  next_run_at?: string | null;
  created_at: string;
}

export interface RpaTaskStepDto {
  uuid: string;
  step_type: string;
  name: string;
  config: Record<string, unknown>;
  enabled?: boolean | null;
  position_x?: number | null;
  position_y?: number | null;
  sort_order?: number | null;
  next_step_uuid?: string | null;
  branch_config?: Record<string, unknown> | null;
}

export interface RpaTaskDetailDto {
  task: RpaTaskDto;
  steps: RpaTaskStepDto[];
  environment_uuids: string[];
}

export interface RpaTaskInputVariable {
  name: string;
  value: string;
  promptOnRun: boolean;
  required: boolean;
}

export interface PaginatedRpaTaskList {
  items: RpaTask[];
  total: number;
  page: number;
  page_size: number;
}

export interface EnvironmentListItem {
  id: string;
  name: string;
  status: 'running' | 'ready';
}

export interface PaginatedEnvironmentList {
  items: EnvironmentListItem[];
  total: number;
  page: number;
  page_size: number;
}

interface EnvironmentDto {
  environment?: {
    id?: string | number;
    uuid?: string;
    name?: string;
    status?: string;
  };
}

export interface UpsertRpaTaskRequest {
  uuid?: string;
  name: string;
  description?: string;
  tags?: string[];
  trigger_type: string;
  schedule?: string;
  cron_expression?: string;
  run_mode: string;
  retry_count?: number;
  retry_interval?: number;
  timeout?: number;
  concurrency?: number;
  stop_on_error?: boolean;
  notify_on_complete?: boolean;
  notify_on_error?: boolean;
  environment_uuids?: string[];
  steps?: Array<{
    step_type: string;
    name: string;
    config: Record<string, unknown>;
    enabled?: boolean;
    position_x?: number;
    position_y?: number;
    sort_order?: number;
  }>;
}

function toStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.map(String).filter(Boolean);
}

interface StoredWorkflowMetaVariable {
  name?: string;
  value?: string;
  prompt_on_run?: boolean;
  required?: boolean;
}

interface StoredWorkflowMeta {
  start_step_id?: string | null;
  global_variables?: StoredWorkflowMetaVariable[];
}

function normalizeStatus(status: string): RpaTask['status'] {
  if (status === 'running' || status === 'completed' || status === 'failed') {
    return status;
  }
  return 'idle';
}

export function transformRpaTaskDto(dto: RpaTaskDto): RpaTask {
  const runCount = dto.run_count ?? 0;
  const successCount = dto.success_count ?? 0;

  return {
    id: dto.uuid,
    name: dto.name,
    description: dto.description ?? undefined,
    triggerType: (dto.trigger_type as RpaTask['triggerType']) || 'manual',
    status: normalizeStatus(dto.status),
    environmentCount: dto.environment_count ?? 0,
    lastRunAt: dto.last_run_at ?? undefined,
    nextRunAt: dto.next_run_at ?? undefined,
    schedule: dto.schedule ?? undefined,
    createdAt: dto.created_at,
    runCount,
    successRate: runCount > 0 ? Math.round((successCount / runCount) * 100) : 0,
  };
}

export async function listRpaTasksPage(request?: {
  page?: number;
  page_size?: number;
  filters?: {
    keyword?: string;
    status?: string;
    trigger_type?: string;
    tags?: string[];
  };
}): Promise<PaginatedRpaTaskList> {
  const page = request?.page ?? 1;
  const pageSize = request?.page_size ?? 20;
  const filters = request?.filters;

  const result = await post<{
    items?: RpaTaskDto[];
    total?: number;
    page?: number;
    page_size?: number;
  }>(API_ENDPOINTS.LIST_TASKS, {
    page,
    page_size: pageSize,
    filters,
  });

  if (!isSuccess(result)) {
    throw new Error(result.message || '获取 RPA 任务列表失败');
  }

  const items = (result.data?.items || []).map(transformRpaTaskDto);

  return {
    items,
    total: result.data?.total ?? items.length,
    page: result.data?.page ?? page,
    page_size: result.data?.page_size ?? pageSize,
  };
}

export async function listRpaTasks(request?: {
  page?: number;
  page_size?: number;
  filters?: {
    keyword?: string;
    status?: string;
    trigger_type?: string;
    tags?: string[];
  };
}): Promise<RpaTask[]> {
  const result = await listRpaTasksPage(request);
  return result.items;
}

export async function getRpaTaskDetail(uuid: string): Promise<RpaTaskDetailDto> {
  const result = await post<RpaTaskDetailDto>(API_ENDPOINTS.GET_TASK_DETAIL, { uuid });
  if (!isSuccess(result) || !result.data) {
    throw new Error(result.message || '获取 RPA 任务详情失败');
  }
  return result.data;
}

export async function createRpaTask(data: UpsertRpaTaskRequest): Promise<string> {
  const result = await post<{ uuid: string }>(API_ENDPOINTS.CREATE_TASK, data);
  if (!isSuccess(result) || !result.data?.uuid) {
    throw new Error(result.message || '创建 RPA 任务失败');
  }
  return result.data.uuid;
}

export async function updateRpaTask(data: UpsertRpaTaskRequest & { uuid: string }): Promise<void> {
  const result = await post(API_ENDPOINTS.UPDATE_TASK, data);
  if (!isSuccess(result)) {
    throw new Error(result.message || '更新 RPA 任务失败');
  }
}

export async function deleteRpaTask(uuid: string): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_TASK, { uuid });
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除 RPA 任务失败');
  }
}

export async function batchDeleteRpaTasks(uuids: string[]): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_DELETE_TASKS, { uuids });
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量删除 RPA 任务失败');
  }
}

export async function duplicateRpaTask(uuid: string): Promise<string> {
  const result = await post<{ uuid: string }>(API_ENDPOINTS.DUPLICATE_TASK, { uuid });
  if (!isSuccess(result) || !result.data?.uuid) {
    throw new Error(result.message || '复制 RPA 任务失败');
  }
  return result.data.uuid;
}

export async function listRpaEnvironments(): Promise<EnvironmentListItem[]> {
  const result = await post<{ items?: EnvironmentDto[] }>(API_ENDPOINTS.LIST_ENVIRONMENTS, {
    page: 1,
    page_size: 1000,
  });

  if (!isSuccess(result)) {
    throw new Error(result.message || '获取环境列表失败');
  }

  return (result.data?.items || []).map((item) => ({
    id: String(item.environment?.uuid || item.environment?.id || ''),
    name: String(item.environment?.name || ''),
    status: item.environment?.status === 'running' ? 'running' : 'ready',
  }));
}

export async function listRpaEnvironmentsPage(request?: {
  page?: number;
  page_size?: number;
  keyword?: string;
}): Promise<PaginatedEnvironmentList> {
  const page = request?.page ?? 1;
  const pageSize = request?.page_size ?? 20;
  const keyword = request?.keyword?.trim();

  const result = await post<{
    items?: EnvironmentDto[];
    total?: number;
    page?: number;
    page_size?: number;
  }>(API_ENDPOINTS.LIST_ENVIRONMENTS, {
    page,
    page_size: pageSize,
    ...(keyword ? { keyword } : {}),
  });

  if (!isSuccess(result)) {
    throw new Error(result.message || '获取环境列表失败');
  }

  const items = (result.data?.items || []).map((item) => ({
    id: String(item.environment?.uuid || item.environment?.id || ''),
    name: String(item.environment?.name || ''),
    status: item.environment?.status === 'running' ? 'running' : 'ready',
  }));

  return {
    items,
    total: result.data?.total ?? items.length,
    page: result.data?.page ?? page,
    page_size: result.data?.page_size ?? pageSize,
  };
}

function extractStoredWorkflowMeta(detail: RpaTaskDetailDto): StoredWorkflowMeta | null {
  const orderedSteps = [...detail.steps].sort(
    (a, b) => (a.sort_order ?? Number.MAX_SAFE_INTEGER) - (b.sort_order ?? Number.MAX_SAFE_INTEGER)
  );

  const workflowMeta = orderedSteps.find((step) => step.config?.workflow_meta)?.config?.workflow_meta;
  if (!workflowMeta || typeof workflowMeta !== 'object') {
    return null;
  }

  return workflowMeta as StoredWorkflowMeta;
}

export function extractTaskInputVariables(detail: RpaTaskDetailDto): RpaTaskInputVariable[] {
  const workflowMeta = extractStoredWorkflowMeta(detail);
  if (!Array.isArray(workflowMeta?.global_variables)) {
    return [];
  }

  return workflowMeta.global_variables
    .map((variable) => ({
      name: typeof variable?.name === 'string' ? variable.name.trim() : '',
      value: typeof variable?.value === 'string' ? variable.value : '',
      promptOnRun: variable?.prompt_on_run === true,
      required: variable?.required === true,
    }))
    .filter((variable) => variable.name && variable.promptOnRun);
}

export function extractWorkflowSchema(detail: RpaTaskDetailDto): RpaWorkflowSchema | null {
  const tags = toStringArray(detail.task.tags);
  const orderedSteps = [...detail.steps].sort(
    (a, b) => (a.sort_order ?? Number.MAX_SAFE_INTEGER) - (b.sort_order ?? Number.MAX_SAFE_INTEGER)
  );
  const workflowMeta = extractStoredWorkflowMeta(detail);

  const workflowSteps = orderedSteps
    .map((step, index) => {
      const storedStep = step.config?.workflow_step;
      if (storedStep && typeof storedStep === 'object') {
        return storedStep;
      }

      return {
        id: step.uuid,
        type: step.step_type,
        name: step.name,
        enabled: step.enabled ?? true,
        next: orderedSteps[index + 1]?.uuid ?? null,
        ui: {
          position: {
            x: step.position_x ?? 0,
            y: step.position_y ?? 0,
          },
        },
        config: step.config ?? {},
      };
    })
    .filter(Boolean);

  return {
    schema_version: '1.0',
    name: detail.task.name,
    description: detail.task.description ?? '',
    tags,
    trigger: {
      type:
        detail.task.trigger_type === 'scheduled' || detail.task.trigger_type === 'event'
          ? detail.task.trigger_type
          : 'manual',
      schedule: detail.task.schedule ?? undefined,
      cron_expression: detail.task.cron_expression ?? undefined,
    },
    environments: detail.environment_uuids || [],
    settings: {
      run_mode: detail.task.run_mode === 'parallel' ? 'parallel' : 'sequential',
      retry_count: detail.task.retry_count ?? 3,
      retry_interval_seconds: detail.task.retry_interval ?? 5,
      timeout_seconds: detail.task.timeout ?? 300,
      concurrency: detail.task.concurrency ?? 1,
      stop_on_error: detail.task.stop_on_error ?? true,
      notify_on_complete: detail.task.notify_on_complete ?? false,
      notify_on_error: detail.task.notify_on_error ?? true,
    },
    variables: Array.isArray(workflowMeta?.global_variables)
      ? workflowMeta.global_variables
          .map((variable) => ({
            name: typeof variable?.name === 'string' ? variable.name.trim() : '',
            value: typeof variable?.value === 'string' ? variable.value : '',
          }))
          .filter((variable) => variable.name)
      : [],
    start_step_id:
      typeof workflowMeta?.start_step_id === 'string'
        ? workflowMeta.start_step_id
        : workflowSteps[0]?.id ?? null,
    steps: workflowSteps as RpaWorkflowSchema['steps'],
  };
}

