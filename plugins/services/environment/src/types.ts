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
  signature?: string;
  arch?: string;
  package_format?: string;
  requires_extract?: boolean;
  entrypoint_template?: string;
  extract_root?: string;
  status?: string;
  is_latest?: boolean;
  installed?: boolean;
}

export interface ProxyPassword {
  value: string;
  encrypted: boolean;
}

export interface ProxyConfig {
  host: string;
  port: number;
  proxy_type: string;
  username?: string;
  password?: ProxyPassword;
}

export interface EnvironmentProxyLike {
  host?: string;
  port?: number;
  proxy_type?: string;
  username?: string;
  password?: string;
}

export interface EnvironmentAccountLike {
  account: string;
  password?: string;
  platform_url: string;
  platform_name?: string;
}

export interface EnvironmentExtensionLike {
  extension_id: string;
  name: string;
  version: string;
  download_url?: string;
  hash?: string;
  icon_url?: string;
}

export interface EnvironmentUrlLike {
  id: number;
  environment_uuid: string;
  url: string;
  title?: string;
  sort_order?: number;
  created_at: string;
}

export interface EnvironmentCookieLike {
  site: string;
  cookie_text: string;
}

export interface EnvironmentInfoLike {
  id?: number;
  uuid: string;
  name?: string;
}

export interface EnvironmentLaunchDetail {
  environment?: EnvironmentInfoLike;
  config?: Record<string, unknown>;
  cookies?: EnvironmentCookieLike[];
  urls?: EnvironmentUrlLike[];
  proxy?: EnvironmentProxyLike | null;
  accounts?: EnvironmentAccountLike[];
  extensions?: EnvironmentExtensionLike[];
}

export interface DefaultStoragePaths {
  profiles: string;
  cache: string;
}

export interface KernelPrepareStatusPayload {
  env_uuid: string | null;
  kernel_value: string;
  status: string;
  message?: string;
}

export interface EnvConnectionPayload {
  env_id: string;
  status: 'connected' | 'disconnected';
}

export type EnvironmentStatus =
  | 'initializing'
  | 'verifying'
  | 'downloading'
  | 'extracting'
  | 'ready'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'stopped'
  | 'error';

export interface BatchLaunchRequest {
  exe_path: string;
  env_uuid: string;
  cache_path: string;
  proxy?: ProxyConfig | null;
  fingerprint_config?: Record<string, unknown> | null;
  accounts?: EnvironmentAccountLike[];
  extensions?: EnvironmentExtensionLike[];
}

export interface BatchLaunchResult {
  env_uuid: string;
  success: boolean;
  error?: string;
}

