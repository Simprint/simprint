/**
 * 环境管理 API
 */
import { invoke } from '@/lib/tauri';
import { post, isSuccess } from '@/lib/request';
import { ITEMS_PER_PAGE } from '../constants';
import type {
  ListEnvironmentsRequest,
  GetEnvironmentRequest,
  CreateEnvironmentRequest,
  UpdateEnvironmentRequest,
  DeleteEnvironmentRequest,
  SetEnvironmentProxyRequest,
  SetEnvironmentAccountsRequest,
  AssignTagsRequest,
  BatchAssignTagsRequest,
  RemoveTagRequest,
  BatchRemoveTagsRequest,
  MoveToGroupRequest,
  BatchMoveToGroupRequest,
  BatchDeleteRequest,
  ExportEnvironmentsRequest,
  ImportEnvironmentsRequest,
  EnvironmentListResponse,
  EnvironmentDetailResponse,
  CreateResponse,
  CookieGroupItem,
  UrlItem,
  Environment,
  GroupItem,
  TagItem,
} from './index.types';

export * from './index.types';

// ============ 类型声明 ============

/** 浏览器内核版本（用于启动时解析下载信息） */
export interface BrowserKernelVersion {
  kernel_id: string;
  type_code: string;
  resource_name: string;
  version: string;
  name?: string;
  notes?: string;
  platform?: string;
  url?: string;
  /** zip 包 SHA256，用于下载后校验 */
  hash?: string;
  /** simprint.exe SHA256，用于校验可执行文件是否被篡改（约定：前 10MB 的 SHA256） */
  signature?: string;
  arch?: string;
  package_format?: string;
  requires_extract?: boolean;
  entrypoint_template?: string;
  extract_root?: string;
  installed?: boolean;
}

/** 代理项 */
export interface ProxyItem {
  id: number;
  uuid: string;
  name: string;
  host: string;
  port: number;
  proxy_type: string;
  username?: string;
  password?: string; // 密码
  country?: string;
  city?: string;
  remark?: string;
  status: string;
  latency?: number;
  environments_count?: number;
}

export interface TauriProxyConfig {
  host: string;
  port: number;
  proxy_type: string;
  username?: string;
  password?:
    | {
        value: string;
        encrypted: boolean;
      }
    | undefined;
}

/** 账号项 */
export interface AccountItem {
  id: number;
  uuid: string;
  platform_url: string;
  platform_name?: string;
  account: string;
  status: string;
  remark?: string;
}

// ============ 常量 ============

export const SIMPRINT_KERNEL_CHROMIUM = 'SIMPRINT_KERNEL_CHROMIUM';

/** 将 system 转为 list 接口的 platform */
export const systemToPlatform: Record<string, string> = {
  Windows: 'windows',
  macOS: 'darwin',
  Linux: 'linux',
};

// ============ API 端点 ============

export const API_ENDPOINTS = {
  // 环境
  LIST_ENVIRONMENTS: 'environments/list',
  GET_ENVIRONMENT: 'environments/detail',
  BATCH_GET_ENVIRONMENTS: 'environments/batch-detail',
  CREATE_ENVIRONMENT: 'environments/create',
  UPDATE_ENVIRONMENT: 'environments/update',
  DELETE_ENVIRONMENT: 'environments/delete',
  BATCH_DELETE_ENVIRONMENTS: 'environments/batch-delete',
  SET_PROXY: 'environments/set-proxy',
  SET_ACCOUNTS: 'environments/set-accounts',
  ASSIGN_TAGS: 'environments/assign-tags',
  BATCH_ASSIGN_TAGS: 'environments/batch-assign-tags',
  REMOVE_TAG: 'environments/remove-tag',
  BATCH_REMOVE_TAGS: 'environments/batch-remove-tags',
  MOVE_TO_GROUP: 'environments/move-to-group',
  BATCH_MOVE_TO_GROUP: 'environments/batch-move-to-group',
  EXPORT_ENVIRONMENTS: 'environments/export',
  IMPORT_ENVIRONMENTS: 'environments/import',

  // 回收站
  LIST_RECYCLE_BIN: 'environments/recycle-bin/list',
  RESTORE_ENVIRONMENT: 'environments/recycle-bin/restore',
  BATCH_RESTORE_ENVIRONMENTS: 'environments/recycle-bin/batch-restore',
  PERMANENT_DELETE_ENVIRONMENT: 'environments/recycle-bin/permanent-delete',
  BATCH_PERMANENT_DELETE_ENVIRONMENTS: 'environments/recycle-bin/batch-permanent-delete',

  // 分组
  LIST_GROUPS: 'groups/list',
  CREATE_GROUP: 'groups/create',
  UPDATE_GROUP: 'groups/update',
  DELETE_GROUP: 'groups/delete',

  // 浏览器内核（用于启动环境）
  LIST_BROWSER_KERNELS: 'browser-kernels/list',

  // 标签
  LIST_TAGS: 'tags/list',
  CREATE_TAG: 'tags/create',
  UPDATE_TAG: 'tags/update',

  // Cookies
  LIST_COOKIES: 'environments/cookies/list',
  ADD_COOKIES: 'environments/cookies/add',
  CLEAR_COOKIES: 'environments/cookies/clear',

  // URLs
  LIST_URLS: 'environments/urls/list',
  ADD_URL: 'environments/urls/add',
  DELETE_URL: 'environments/urls/delete',
  CLEAR_URLS: 'environments/urls/clear',

  // 代理
  LIST_PROXIES: 'proxies/list',
  CREATE_PROXY: 'proxies/create',
  DELETE_PROXY: 'proxies/delete',

  // 账号
  LIST_ACCOUNTS: 'accounts/list',
  CREATE_ACCOUNT: 'accounts/create',
  DELETE_TAG: 'tags/delete',
} as const;

// ============ 工具函数 ============

/**
 * 将后端 DTO 转换为前端 Environment 格式
 * 后端返回完整的关联对象数据
 */
function transformEnvironmentDto(dto: any): Environment {
  return {
    id: dto.id,
    uuid: dto.uuid || '',
    name: dto.name || '',
    description: dto.description,
    status: dto.status || 'ready',
    system_info: dto.system_info,
    kernel_info: dto.kernel_info,
    fingerprint_summary: dto.fingerprint_summary,
    last_opened_at: dto.last_opened_at,
    created_at: dto.created_at,
    updated_at: dto.updated_at,
    // 分组详情（完整对象）
    group: dto.group || undefined,
    // 代理详情（完整对象）
    proxy: dto.proxy || undefined,
    // 标签列表（完整对象列表）
    tags: dto.tags || [],
    // 账号列表（完整对象列表）
    accounts: dto.accounts || [],
    // 扩展列表
    extensions: dto.extensions || [],
  };
}

/**
 * 将后端 GroupDto 转换为前端 GroupItem 格式
 */
function transformGroupDto(dto: any): GroupItem {
  return {
    id: String(dto.id || ''),
    uuid: dto.uuid || '',
    name: dto.name || '',
    description: dto.description,
  };
}

/**
 * 将后端 TagDto 转换为前端 TagItem 格式
 */
function transformTagDto(dto: any): TagItem {
  return {
    id: String(dto.id || ''),
    uuid: dto.uuid || '',
    name: dto.name || '',
    color: dto.color || '',
    environmentsCount: dto.environments_count,
  };
}

// ============ 环境 API ============

/**
 * 获取环境列表
 */
export async function listEnvironments(
  request: ListEnvironmentsRequest = {}
): Promise<EnvironmentListResponse> {
  const result = await post<{
    items: any[];
    total: number;
    page: number;
    page_size: number;
  }>(
    API_ENDPOINTS.LIST_ENVIRONMENTS,
    {
      page: request.page || 1,
      page_size: request.page_size || ITEMS_PER_PAGE,
      filters: request.filters,
    }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取环境列表失败');
  }
  const data = result.data!;
  // 后端返回的是 EnvironmentDetailResponse 列表，需要提取 environment 并合并 group、proxy、tags、accounts
  return {
    items: (data.items || []).map((item: any) => {
      // item 是 EnvironmentDetailResponse，需要转换为 Environment
      const env = transformEnvironmentDto(item.environment);
      // 合并 group、proxy、tags、accounts
      if (item.group) {
        env.group = item.group;
      }
      if (item.proxy) {
        env.proxy = item.proxy;
      }
      if (item.tags) {
        env.tags = item.tags;
      }
      if (item.accounts) {
        env.accounts = item.accounts;
      }
      return env;
    }),
    total: data.total || 0,
    page: data.page || 1,
    page_size: data.page_size || ITEMS_PER_PAGE,
  };
}

/**
 * 获取环境详情
 */
export async function getEnvironment(
  request: GetEnvironmentRequest
): Promise<EnvironmentDetailResponse> {
  const result = await post<EnvironmentDetailResponse>(
    API_ENDPOINTS.GET_ENVIRONMENT,
    request
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取环境详情失败');
  }
  return result.data!;
}

/**
 * 批量获取环境详情
 */
export async function batchGetEnvironments(
  uuids: string[]
): Promise<Record<string, EnvironmentDetailResponse>> {
  const result = await post<Record<string, EnvironmentDetailResponse>>(
    API_ENDPOINTS.BATCH_GET_ENVIRONMENTS,
    { uuids }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量获取环境详情失败');
  }
  return result.data!;
}

/**
 * 创建环境
 */
export async function createEnvironment(request: CreateEnvironmentRequest): Promise<string> {
  const result = await post<CreateResponse>(API_ENDPOINTS.CREATE_ENVIRONMENT, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '创建环境失败');
  }
  return result.data!.uuid;
}

/**
 * 更新环境
 */
export async function updateEnvironment(request: UpdateEnvironmentRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.UPDATE_ENVIRONMENT, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '更新环境失败');
  }
}

/**
 * 删除环境
 */
export async function deleteEnvironment(request: DeleteEnvironmentRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_ENVIRONMENT, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除环境失败');
  }
}

/**
 * 批量删除环境
 */
export async function batchDeleteEnvironments(request: BatchDeleteRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_DELETE_ENVIRONMENTS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量删除失败');
  }
}

// ============ 回收站相关 ============

/**
 * 查询回收站环境列表
 */
export async function listRecycleBin(
  request: ListEnvironmentsRequest = {}
): Promise<EnvironmentListResponse> {
  const result = await post<{
    items: any[];
    total: number;
    page: number;
    page_size: number;
  }>(API_ENDPOINTS.LIST_RECYCLE_BIN, {
    page: request.page || 1,
    page_size: request.page_size || ITEMS_PER_PAGE,
    filters: request.filters,
  });
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取回收站列表失败');
  }
  const data = result.data!;
  // 后端返回的是 EnvironmentDetailResponse 列表，需要提取 environment 并合并 group、proxy、tags、accounts
  return {
    items: (data.items || []).map((item: any) => {
      // item 是 EnvironmentDetailResponse，需要转换为 Environment
      const env = transformEnvironmentDto(item.environment);
      // 合并 group、proxy、tags、accounts
      if (item.group) {
        env.group = item.group;
      }
      if (item.proxy) {
        env.proxy = item.proxy;
      }
      if (item.tags) {
        env.tags = item.tags;
      }
      if (item.accounts) {
        env.accounts = item.accounts;
      }
      return env;
    }),
    total: data.total || 0,
    page: data.page || 1,
    page_size: data.page_size || ITEMS_PER_PAGE,
  };
}

/**
 * 恢复环境
 */
export async function restoreEnvironment(uuid: string): Promise<void> {
  const result = await post(API_ENDPOINTS.RESTORE_ENVIRONMENT, { uuid });
  if (!isSuccess(result)) {
    throw new Error(result.message || '恢复环境失败');
  }
}

/**
 * 批量恢复环境
 */
export async function batchRestoreEnvironments(uuids: string[]): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_RESTORE_ENVIRONMENTS, { uuids });
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量恢复失败');
  }
}

/**
 * 永久删除环境
 */
export async function permanentDeleteEnvironment(uuid: string): Promise<void> {
  const result = await post(API_ENDPOINTS.PERMANENT_DELETE_ENVIRONMENT, { uuid });
  if (!isSuccess(result)) {
    throw new Error(result.message || '永久删除失败');
  }
}

/**
 * 批量永久删除环境
 */
export async function batchPermanentDeleteEnvironments(uuids: string[]): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_PERMANENT_DELETE_ENVIRONMENTS, { uuids });
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量永久删除失败');
  }
}

// ============ 环境操作 ============

/**
 * 设置环境代理
 */
export async function setEnvironmentProxy(request: SetEnvironmentProxyRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.SET_PROXY, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '设置代理失败');
  }
}

/**
 * 设置环境账号
 */
export function buildTauriProxyConfig(proxy?: ProxyItem | null): TauriProxyConfig | null {
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

export async function refreshEnvironmentProxy(
  envUuid: string,
  proxy?: ProxyItem | null
): Promise<void> {
  await invoke('refresh_environment_proxy', {
    envUuid,
    proxy: buildTauriProxyConfig(proxy),
  });
}

export async function setEnvironmentAccounts(
  request: SetEnvironmentAccountsRequest
): Promise<void> {
  const result = await post(API_ENDPOINTS.SET_ACCOUNTS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '设置账号失败');
  }
}

/**
 * 分配标签
 */
export async function assignTags(request: AssignTagsRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.ASSIGN_TAGS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '分配标签失败');
  }
}

/**
 * 批量分配标签
 */
export async function batchAssignTags(request: BatchAssignTagsRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_ASSIGN_TAGS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量分配标签失败');
  }
}

/**
 * 移除标签
 */
export async function removeTag(request: RemoveTagRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.REMOVE_TAG, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '移除标签失败');
  }
}

/**
 * 批量移除标签
 */
export async function batchRemoveTags(request: BatchRemoveTagsRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_REMOVE_TAGS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量移除标签失败');
  }
}

/**
 * 移动到分组
 */
export async function moveToGroup(request: MoveToGroupRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.MOVE_TO_GROUP, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '移动到分组失败');
  }
}

/**
 * 批量移动到分组
 */
export async function batchMoveToGroup(request: BatchMoveToGroupRequest): Promise<void> {
  const result = await post(API_ENDPOINTS.BATCH_MOVE_TO_GROUP, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '批量移动到分组失败');
  }
}

/**
 * 导出环境
 */
export async function exportEnvironments(
  request: ExportEnvironmentsRequest
): Promise<Environment[]> {
  const result = await post<Environment[]>(API_ENDPOINTS.EXPORT_ENVIRONMENTS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '导出环境失败');
  }
  return result.data!;
}

/**
 * 导入环境
 */
export async function importEnvironments(
  request: ImportEnvironmentsRequest
): Promise<Environment[]> {
  const result = await post<Environment[]>(API_ENDPOINTS.IMPORT_ENVIRONMENTS, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '导入环境失败');
  }
  return result.data!;
}

// ============ 浏览器内核 API ============

/**
 * 获取浏览器内核列表（用于启动环境时按 kernel 过滤）
 */
export async function listBrowserKernels(
  platform?: string,
  typeCode?: string
): Promise<Record<string, BrowserKernelVersion[]>> {
  const result = await post<Record<string, BrowserKernelVersion[]>>(
    API_ENDPOINTS.LIST_BROWSER_KERNELS,
    { platform: platform || undefined, type_code: typeCode || undefined }
  );
  if (!isSuccess(result) || !result.data) {
    return {};
  }
  return result.data;
}

// ============ 分组 API ============

/**
 * 获取分组列表
 */
export async function listGroups(): Promise<GroupItem[]> {
  const result = await post<any[]>(API_ENDPOINTS.LIST_GROUPS, {});
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取分组列表失败');
  }
  return (result.data || []).map(transformGroupDto);
}

/**
 * 创建分组
 */
export async function createGroup(request: {
  name: string;
  description?: string;
}): Promise<string> {
  const result = await post<CreateResponse>(API_ENDPOINTS.CREATE_GROUP, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '创建分组失败');
  }
  return result.data!.uuid;
}

/**
 * 更新分组
 */
export async function updateGroup(request: {
  uuid: string;
  name?: string;
  description?: string;
}): Promise<void> {
  const result = await post(API_ENDPOINTS.UPDATE_GROUP, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '更新分组失败');
  }
}

/**
 * 删除分组
 */
export async function deleteGroup(request: { uuid: string }): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_GROUP, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除分组失败');
  }
}

// ============ 标签 API ============

/**
 * 获取标签列表
 */
export async function listTags(): Promise<TagItem[]> {
  const result = await post<any[]>(API_ENDPOINTS.LIST_TAGS, {});
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取标签列表失败');
  }
  return (result.data || []).map(transformTagDto);
}

/**
 * 创建标签
 */
export async function createTag(request: { name: string; color: string }): Promise<TagItem> {
  const result = await post<any>(API_ENDPOINTS.CREATE_TAG, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '创建标签失败');
  }
  return transformTagDto(result.data);
}

/**
 * 更新标签
 */
export async function updateTag(request: {
  uuid: string;
  name?: string;
  color?: string;
}): Promise<void> {
  const result = await post(API_ENDPOINTS.UPDATE_TAG, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '更新标签失败');
  }
}

/**
 * 删除标签
 */
export async function deleteTag(request: { uuid: string }): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_TAG, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除标签失败');
  }
}

// ============ 代理 API ============

/**
 * 获取代理列表
 *
 * 注意：密码字段现在直接为明文。
 */
export async function listProxies(): Promise<ProxyItem[]> {
  const result = await post<{ items: any[] }>(
    API_ENDPOINTS.LIST_PROXIES,
    {
      page: 1,
      page_size: 1000,
    }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取代理列表失败');
  }
  return (result.data?.items || []).map((dto: any) => ({
    id: dto.id,
    uuid: dto.uuid || '',
    name: dto.name || '',
    host: dto.host || '',
    port: dto.port || 0,
    proxy_type: dto.proxy_type || 'http',
    username: dto.username,
    password: dto.password,
    country: dto.country,
    city: dto.city,
    remark: dto.remark,
    status: dto.status || 'unknown',
    latency: dto.latency,
    environments_count: dto.environments_count,
  }));
}

/**
 * 创建代理
 */
export async function createProxy(request: {
  name: string;
  host: string;
  port: number;
  proxy_type: string;
  username?: string;
  password?: string;
  country?: string;
  city?: string;
  ssh_key?: string;
  ssh_passphrase?: string;
}): Promise<string> {
  const result = await post<{ uuid: string }>(API_ENDPOINTS.CREATE_PROXY, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '创建代理失败');
  }
  return result.data!.uuid;
}

/**
 * 删除代理
 */
export async function deleteProxy(request: { uuid: string }): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_PROXY, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除代理失败');
  }
}

// ============ 账号 API ============

/**
 * 获取账号列表
 */
export async function listAccounts(): Promise<AccountItem[]> {
  const result = await post<{ items: any[] }>(
    API_ENDPOINTS.LIST_ACCOUNTS,
    {
      page: 1,
      page_size: 1000,
    }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取账号列表失败');
  }
  return (result.data?.items || []).map((dto: any) => ({
    id: dto.id,
    uuid: dto.uuid || '',
    platform_url: dto.platform_url || '',
    platform_name: dto.platform_name,
    account: dto.account || '',
    status: dto.status || 'active',
    remark: dto.remark,
  }));
}

/**
 * 创建账号
 */
export async function createAccount(request: {
  platform_url: string;
  platform_name?: string;
  account: string;
  password?: string;
  remark?: string;
}): Promise<string> {
  const result = await post<{ uuid: string }>(API_ENDPOINTS.CREATE_ACCOUNT, request);
  if (!isSuccess(result)) {
    throw new Error(result.message || '创建账号失败');
  }
  return result.data!.uuid;
}

// ============ Cookies API ============

/**
 * 获取环境 Cookies
 */
export async function getEnvironmentCookies(uuid: string): Promise<CookieGroupItem[]> {
  const result = await post<CookieGroupItem[]>(
    API_ENDPOINTS.LIST_COOKIES,
    { uuid }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取 Cookies 失败');
  }
  return result.data || [];
}

/**
 * 添加单个环境 Cookie
 */
export async function addEnvironmentCookie(
  uuid: string,
  cookie: CookieGroupItem
): Promise<number> {
  const result = await post<{ id: number }>(API_ENDPOINTS.ADD_COOKIES, {
    environment_uuid: uuid,
    site: cookie.site,
    cookie_text: cookie.cookie_text,
  });
  if (!isSuccess(result)) {
    throw new Error(result.message || '添加 Cookie 失败');
  }
  return result.data?.id || 0;
}

/**
 * 批量添加环境 Cookies（循环调用单个添加接口）
 */
export async function addEnvironmentCookies(
  uuid: string,
  cookies: CookieGroupItem[]
): Promise<void> {
  for (const cookie of cookies) {
    await addEnvironmentCookie(uuid, cookie);
  }
}

/**
 * 清空环境 Cookies
 */
export async function clearEnvironmentCookies(uuid: string): Promise<void> {
  const result = await post(API_ENDPOINTS.CLEAR_COOKIES, { environment_uuid: uuid });
  if (!isSuccess(result)) {
    throw new Error(result.message || '清空 Cookies 失败');
  }
}

// ============ URLs API ============

/**
 * 获取环境 URL 列表
 */
export async function listEnvironmentUrls(uuid: string): Promise<UrlItem[]> {
  const result = await post<UrlItem[]>(
    API_ENDPOINTS.LIST_URLS,
    { uuid }
  );
  if (!isSuccess(result)) {
    throw new Error(result.message || '获取 URL 列表失败');
  }
  return result.data || [];
}

/**
 * 添加环境 URL
 */
export async function addEnvironmentUrl(
  uuid: string,
  url: string,
  title?: string,
  sortOrder?: number
): Promise<number> {
  const result = await post<{ id: number }>(API_ENDPOINTS.ADD_URL, {
    environment_uuid: uuid,
    url,
    title,
    sort_order: sortOrder,
  });
  if (!isSuccess(result)) {
    throw new Error(result.message || '添加 URL 失败');
  }
  return result.data?.id || 0;
}

/**
 * 删除环境 URL
 */
export async function deleteEnvironmentUrl(id: number): Promise<void> {
  const result = await post(API_ENDPOINTS.DELETE_URL, { id });
  if (!isSuccess(result)) {
    throw new Error(result.message || '删除 URL 失败');
  }
}

/**
 * 清空环境 URL
 */
export async function clearEnvironmentUrls(uuid: string): Promise<void> {
  const result = await post(API_ENDPOINTS.CLEAR_URLS, { environment_uuid: uuid });
  if (!isSuccess(result)) {
    throw new Error(result.message || '清空 URL 失败');
  }
}

// ============ 批量启动 API ============

/**
 * 批量启动请求
 */
export interface BatchLaunchRequest {
  exe_path: string;
  env_uuid: string;
  cache_path: string;
  proxy?: {
    host: string;
    port: number;
    proxy_type: string;
    username?: string;
    password?: string;
  } | null;
  fingerprint_config?: Record<string, any> | null;
}

/**
 * 批量启动结果
 */
export interface BatchLaunchResult {
  env_uuid: string;
  success: boolean;
  error?: string;
}
