/**
 * 创建窗口 API 服务
 */
import { post, isSuccess } from '@/lib/request';
import type { CookieGroup, WindowConfig } from '../types';
import type {
  CookieGroupInput,
  CreateEnvironmentRequest,
  CreateEnvironmentResponse,
  CreateEnvironmentOptions,
  BatchCreateEnvironmentRequest,
  BatchCreateEnvironmentResponse,
  CreateTemplateRequest,
  CreateTemplateResponse,
} from './index.types';

// 导出类型
export * from './index.types';

// API 端点配置
export const API_ENDPOINTS = {
  CREATE_ENVIRONMENT: 'environments/create',
  BATCH_CREATE_ENVIRONMENTS: 'environments/batch-create',
  CREATE_TEMPLATE: 'templates/create',
  UPDATE_TEMPLATE: 'templates/update',
  LIST_GROUPS: 'groups/list',
  LIST_TAGS: 'tags/list',
  LIST_BROWSER_KERNELS: 'browser-kernels/list',
} as const;

/** 浏览器内核版本 */
export interface BrowserKernelVersion {
  kernel_id: string;
  type_code: string;
  resource_name: string;
  version: string;
  name?: string;
  notes?: string;
  platform?: string;
  url?: string;
  hash?: string;
  arch?: string;
  package_format?: string;
  requires_extract?: boolean;
  entrypoint_template?: string;
  extract_root?: string;
  installed?: boolean;
}

function buildEnvironmentUrls(urls: string[]) {
  return urls
    .map((url, index) => ({
      url: url.trim(),
      title: undefined,
      sort_order: index,
    }))
    .filter((item) => item.url.length > 0);
}

function buildEnvironmentCookies(cookies: CookieGroup[]): CookieGroupInput[] {
  return cookies
    .map((item) => ({
      site: item.site.trim(),
      cookie_text: item.cookieText.trim(),
    }))
    .filter((item) => item.site.length > 0 && item.cookie_text.length > 0);
}

function buildWindowInfoPayload(config: WindowConfig['windowInfo']) {
  return {
    name: config.name,
    system: config.system,
    kernel: config.kernel,
    kernel_id: config.kernelId || undefined,
    userAgent: config.userAgent,
    searchEngine: config.searchEngine,
    description: config.description,
  };
}

function buildTemplateEnvironmentData(
  config: WindowConfig,
  name: string,
  description?: string,
  options?: {
    group?: {
      id: number;
      uuid: string;
      name: string;
      description?: string;
      sort_order?: number;
    };
    tags?: Array<{
      id: number;
      uuid: string;
      name: string;
      color?: string;
      sort_order?: number;
    }>;
    accounts?: Array<{
      id: number;
      uuid: string;
      platform_url: string;
      platform_name?: string;
      account: string;
      status: string;
      remark?: string;
    }>;
    proxy?: {
      id: number;
      uuid: string;
      name: string;
      host: string;
      port: number;
      proxy_type: string;
      country?: string;
      city?: string;
      status: string;
      latency?: number;
    };
  }
) {
  const now = new Date().toISOString();
  const environmentUuid = '00000000-0000-0000-0000-000000000000';

  return {
    environment: {
      id: 0,
      uuid: environmentUuid,
      user_uuid: environmentUuid,
      team_uuid: null,
      name,
      description: description || null,
      status: 'active',
      group_uuid: options?.group?.uuid || null,
      proxy_uuid: options?.proxy?.uuid || null,
      system_info: config.windowInfo.system || null,
      kernel_info: config.windowInfo.kernel || null,
      fingerprint_summary: null,
      last_opened_at: null,
      created_at: now,
      updated_at: now,
      deleted_at: null,
    },
    config: {
      id: 0,
      environment_uuid: environmentUuid,
      window_info: buildWindowInfoPayload(config.windowInfo),
      basic_settings: config.basicSettings,
      fingerprint_settings: config.advancedFingerprintSettings,
      device_settings: config.deviceSettings,
      preference_settings: config.preferenceSettings,
      project_metadata: config.projectMetadata,
      created_at: now,
      updated_at: now,
    },
    cookies: buildEnvironmentCookies(config.windowInfo.cookies),
    urls: buildEnvironmentUrls(config.windowInfo.urls).map((item) => ({
      id: 0,
      environment_uuid: environmentUuid,
      url: item.url,
      title: item.title || null,
      sort_order: item.sort_order,
      created_at: now,
    })),
    tags: (options?.tags || []).map((tag) => ({
      id: tag.id,
      uuid: tag.uuid,
      user_uuid: environmentUuid,
      team_uuid: null,
      name: tag.name,
      color: tag.color || null,
      sort_order: tag.sort_order || null,
      environments_count: null,
      created_at: now,
      updated_at: now,
      deleted_at: null,
    })),
    accounts: (options?.accounts || []).map((account) => ({
      id: account.id,
      uuid: account.uuid,
      user_uuid: environmentUuid,
      team_uuid: null,
      platform_url: account.platform_url,
      platform_name: account.platform_name || null,
      account: account.account,
      password: null,
      status: account.status,
      remark: account.remark || null,
      usage_count: null,
      last_used_at: null,
      created_at: now,
      updated_at: now,
      deleted_at: null,
    })),
    group: options?.group
      ? {
          id: options.group.id,
          uuid: options.group.uuid,
          name: options.group.name,
          description: options.group.description || null,
          sort_order: options.group.sort_order || null,
        }
      : null,
    proxy: options?.proxy
      ? {
          id: options.proxy.id,
          uuid: options.proxy.uuid,
          name: options.proxy.name,
          host: options.proxy.host,
          port: options.proxy.port,
          proxy_type: options.proxy.proxy_type,
          country: options.proxy.country || null,
          city: options.proxy.city || null,
          status: options.proxy.status,
          latency: options.proxy.latency || null,
          last_check_ip: null,
        }
      : null,
  };
}

/**
 * 获取浏览器内核列表
 * @param platform - 平台：windows/darwin/linux
 * @param type_code - 版本类型，如 SIMPRINT_KERNEL_CHROMIUM
 */
export async function listBrowserKernels(
  platform?: string,
  type_code?: string
): Promise<Record<string, BrowserKernelVersion[]>> {
  const result = await post<Record<string, BrowserKernelVersion[]>>(
    API_ENDPOINTS.LIST_BROWSER_KERNELS,
    { platform: platform || undefined, type_code: type_code || undefined }
  );
  if (!isSuccess(result) || !result.data) {
    return {};
  }
  return result.data;
}

/**
 * 将前端 WindowConfig 转换为后端 CreateEnvironmentRequest
 */
export function transformWindowConfigToRequest(
  config: WindowConfig,
  options: CreateEnvironmentOptions
): CreateEnvironmentRequest {
  return {
    name: config.windowInfo.name || '新窗口',
    description: config.windowInfo.description || undefined,
    group_uuid: options.groupUuid,
    tag_uuids: options.tagUuids?.length ? options.tagUuids : undefined,
    account_uuids: options.accountUuids?.length ? options.accountUuids : undefined,
    proxy_uuid: options.proxyUuid,
    cookies: buildEnvironmentCookies(config.windowInfo.cookies),
    urls: buildEnvironmentUrls(config.windowInfo.urls),
    config: {
      window_info: buildWindowInfoPayload(config.windowInfo),
      basic_settings: {
        language: config.basicSettings.language,
        interfaceLanguage: config.basicSettings.interfaceLanguage,
        timezone: config.basicSettings.timezone,
        geolocationPrompt: config.basicSettings.geolocationPrompt,
        geolocation: config.basicSettings.geolocation,
        sound: config.basicSettings.sound,
        images: config.basicSettings.images,
        video: config.basicSettings.video,
        windowSize: config.basicSettings.windowSize,
        windowWidth: config.basicSettings.windowWidth,
        windowHeight: config.basicSettings.windowHeight,
        windowPosition: config.basicSettings.windowPosition,
      },
      fingerprint_settings: {
        resolution: config.advancedFingerprintSettings.resolution,
        colorDepth: config.advancedFingerprintSettings.colorDepth,
        devicePixelRatio: config.advancedFingerprintSettings.devicePixelRatio,
        maxTouchPoints: config.advancedFingerprintSettings.maxTouchPoints,
        fontFingerprint: config.advancedFingerprintSettings.fontFingerprint,
        fontList: config.advancedFingerprintSettings.fontList,
        webrtc: config.advancedFingerprintSettings.webrtc,
        webglImage: config.advancedFingerprintSettings.webglImage,
        webglInfo: config.advancedFingerprintSettings.webglInfo,
        webglVendor: config.advancedFingerprintSettings.webglVendor,
        webglRenderer: config.advancedFingerprintSettings.webglRenderer,
        webgpu: config.advancedFingerprintSettings.webgpu,
        canvas: config.advancedFingerprintSettings.canvas,
        audioContext: config.advancedFingerprintSettings.audioContext,
        speechVoices: config.advancedFingerprintSettings.speechVoices,
        doNotTrack: config.advancedFingerprintSettings.doNotTrack,
        clientRects: config.advancedFingerprintSettings.clientRects,
        mediaDevices: config.advancedFingerprintSettings.mediaDevices,
      },
      device_settings: {
        deviceName: config.deviceSettings.deviceName,
        deviceNameRandom: config.deviceSettings.deviceNameRandom,
        macAddress: config.deviceSettings.macAddress,
        macAddressMode: config.deviceSettings.macAddressMode,
        hardwareConcurrency: config.deviceSettings.hardwareConcurrency,
        deviceMemory: config.deviceSettings.deviceMemory,
        sslFingerprint: config.deviceSettings.sslFingerprint,
        portScanProtection: config.deviceSettings.portScanProtection,
        scanWhitelist: config.deviceSettings.scanWhitelist,
        hardwareAcceleration: config.deviceSettings.hardwareAcceleration,
        disableSandbox: config.deviceSettings.disableSandbox,
        startupParameters: config.deviceSettings.startupParameters,
      },
      preference_settings: {
        syncWindowName: config.preferenceSettings.syncWindowName,
        customBookmarks: config.preferenceSettings.customBookmarks,
        syncBookmarks: config.preferenceSettings.syncBookmarks,
        syncHistory: config.preferenceSettings.syncHistory,
        syncTabs: config.preferenceSettings.syncTabs,
        syncCookies: config.preferenceSettings.syncCookies,
        syncExtensions: config.preferenceSettings.syncExtensions,
        syncSavedPasswords: config.preferenceSettings.syncSavedPasswords,
        syncIndexedDB: config.preferenceSettings.syncIndexedDB,
        syncLocalStorage: config.preferenceSettings.syncLocalStorage,
        deleteCacheBeforeLaunch: config.preferenceSettings.deleteCacheBeforeLaunch,
        deleteCookiesBeforeLaunch: config.preferenceSettings.deleteCookiesBeforeLaunch,
        deleteLocalStorageBeforeLaunch: config.preferenceSettings.deleteLocalStorageBeforeLaunch,
        randomFingerprintOnLaunch: config.preferenceSettings.randomFingerprintOnLaunch,
        showSavePasswordPrompt: config.preferenceSettings.showSavePasswordPrompt,
        stopOpenIfNetworkUnavailable: config.preferenceSettings.stopOpenIfNetworkUnavailable,
        stopOpenIfIpChanges: config.preferenceSettings.stopOpenIfIpChanges,
        stopOpenIfCountryChanges: config.preferenceSettings.stopOpenIfCountryChanges,
        openWorkbench: config.preferenceSettings.openWorkbench,
        ipChangeReminder: config.preferenceSettings.ipChangeReminder,
        enableGoogleLogin: config.preferenceSettings.enableGoogleLogin,
        urlBlacklist: config.preferenceSettings.urlBlacklist,
        urlWhitelist: config.preferenceSettings.urlWhitelist,
      },
      project_metadata: {
        defaultProject: config.projectMetadata.defaultProject,
        tags: config.projectMetadata.tags,
      },
    },
  };
}

/**
 * 创建环境
 */
export async function createEnvironment(
  config: WindowConfig,
  options?: CreateEnvironmentOptions
): Promise<CreateEnvironmentResponse> {
  const requestData = transformWindowConfigToRequest(config, options);
  const result = await post<CreateEnvironmentResponse>(
    API_ENDPOINTS.CREATE_ENVIRONMENT,
    requestData
  );

  if (!isSuccess(result)) {
    throw new Error(result.message || '创建环境失败');
  }

  return result.data!;
}

/**
 * 批量创建环境
 */
export async function batchCreateEnvironments(
  configs: WindowConfig[],
  proxyUuids: string[] | undefined, // 代理 UUID 数组，按顺序分配给环境
  options?: CreateEnvironmentOptions
): Promise<CreateEnvironmentResponse[]> {
  // 构建批量创建请求
  const environments: CreateEnvironmentRequest[] = configs.map((config, index) => {
    const request = transformWindowConfigToRequest(config, {
      // 统一设置分组和标签（批量创建时这些应该相同）
      groupUuid: options?.groupUuid,
      tagUuids: options?.tagUuids?.length ? options.tagUuids : undefined,
      accountUuids:
        config.windowInfo.accountUuids.length > 0 ? config.windowInfo.accountUuids : undefined,
      proxyUuid: undefined, // 代理单独处理
    });

    // 分配代理逻辑：
    // - 如果没有代理，不传递 proxy_uuid
    // - 如果代理数量等于窗口数量，按顺序放入
    // - 如果代理数量少于窗口数量，按顺序放入，缺失的不包含代理
    // - 如果代理数量多于窗口数量，按顺序放入，多出的不处理
    if (proxyUuids && proxyUuids.length > 0 && index < proxyUuids.length) {
      request.proxy_uuid = proxyUuids[index];
    }
    // 如果 index >= proxyUuids.length，则不设置 proxy_uuid（undefined）

    return request;
  });

  const requestData: BatchCreateEnvironmentRequest = {
    environments,
  };

  const result = await post<BatchCreateEnvironmentResponse>(
    API_ENDPOINTS.BATCH_CREATE_ENVIRONMENTS,
    requestData
  );

  if (!isSuccess(result)) {
    throw new Error(result.message || '批量创建环境失败');
  }

  // 后端返回的是 CreateEnvironmentResponse 数组
  return (result.data?.data || result.data || []) as CreateEnvironmentResponse[];
}

/**
 * 保存为模板
 * 构建完整的环境详情数据（EnvironmentDetailResponse 格式）
 */
export async function saveAsTemplate(
  config: WindowConfig,
  name: string,
  description?: string,
  isPublic?: boolean,
  options?: {
    group?: {
      id: number;
      uuid: string;
      name: string;
      description?: string;
      sort_order?: number;
    };
    tags?: Array<{
      id: number;
      uuid: string;
      name: string;
      color?: string;
      sort_order?: number;
    }>;
    accounts?: Array<{
      id: number;
      uuid: string;
      platform_url: string;
      platform_name?: string;
      account: string;
      status: string;
      remark?: string;
    }>;
    proxy?: {
      id: number;
      uuid: string;
      name: string;
      host: string;
      port: number;
      proxy_type: string;
      country?: string;
      city?: string;
      status: string;
      latency?: number;
    };
  }
): Promise<CreateTemplateResponse> {
  // 构建完整的环境详情数据（EnvironmentDetailResponse 格式）
  // 传递完整的数据对象，包含所有必要信息
  const environmentData = buildTemplateEnvironmentData(config, name, description, options);

  const requestData: CreateTemplateRequest = {
    name,
    description,
    is_public: isPublic,
    environment_data: environmentData,
  };

  const result = await post<CreateTemplateResponse>(API_ENDPOINTS.CREATE_TEMPLATE, requestData);

  if (!isSuccess(result)) {
    throw new Error(result.message || '保存模板失败');
  }

  return result.data!;
}

/**
 * 更新模板
 * 构建完整的环境详情数据（EnvironmentDetailResponse 格式）
 */
export async function updateTemplate(
  templateUuid: string,
  config: WindowConfig,
  name: string,
  description?: string,
  isPublic?: boolean,
  options?: {
    group?: {
      id: number;
      uuid: string;
      name: string;
      description?: string;
      sort_order?: number;
    };
    tags?: Array<{
      id: number;
      uuid: string;
      name: string;
      color?: string;
      sort_order?: number;
    }>;
    accounts?: Array<{
      id: number;
      uuid: string;
      platform_url: string;
      platform_name?: string;
      account: string;
      status: string;
      remark?: string;
    }>;
    proxy?: {
      id: number;
      uuid: string;
      name: string;
      host: string;
      port: number;
      proxy_type: string;
      country?: string;
      city?: string;
      status: string;
      latency?: number;
    };
  }
): Promise<void> {
  // 构建完整的环境详情数据（EnvironmentDetailResponse 格式）
  // 注意：environment 对象需要包含 EnvironmentDto 的所有必需字段
  const environmentData = buildTemplateEnvironmentData(config, name, description, options);

  const requestData = {
    uuid: templateUuid,
    name,
    description,
    is_public: isPublic,
    config_json: environmentData, // 后端期望 config_json 字段
  };

  const result = await post(API_ENDPOINTS.UPDATE_TEMPLATE, requestData);

  if (!isSuccess(result)) {
    throw new Error(result.message || '更新模板失败');
  }
}
