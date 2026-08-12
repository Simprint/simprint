/**
 * 窗口创建相关类型定义
 */

export type CreateTab = 'single' | 'batch' | 'import';

export type MatchMode = 'ip' | 'custom';

export type GeolocationPrompt = 'ask' | 'allow' | 'forbid';

export type WebRTCMode = 'replace' | 'real' | 'disable';

export type FingerprintMode = 'random' | 'real' | 'custom' | 'system';

export type ResolutionMode = 'system' | 'random';

/**
 * 屏幕分辨率配置
 */
export interface Resolution {
  width: number;
  height: number;
}

/**
 * 字体列表配置
 */
export interface FontListConfig {
  mode: 'system' | 'random' | 'ua-match' | 'custom';
  fonts?: string[];
}

export type WindowSizeMode = 'custom' | 'fullscreen';

export type PositionQuadrant = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

export type ProxySourceMode = 'remote' | 'local';

export interface CookieGroup {
  site: string;
  cookieText: string;
}

/**
 * 窗口信息
 */
export interface WindowInfo {
  name: string;
  system: string; // 操作系统
  kernel: string; // 浏览器内核显示名称
  kernelId: string; // 数据库中不可变的内核制品标识
  userAgent: string;
  searchEngine: string;
  proxySourceMode: ProxySourceMode;
  proxyUuids: string[]; // 代理 UUID 列表（用于批量创建）
  localProxyNodeNames: string[]; // 本地代理节点名称列表（用于批量创建）
  accountUuids: string[]; // 平台账号 UUID 列表
  urls: string[];
  cookies: CookieGroup[];
  description: string; // 描述
}

/**
 * 基础设置
 */
export interface BasicSettings {
  language: MatchMode;
  interfaceLanguage: MatchMode;
  timezone: MatchMode;
  geolocationPrompt: GeolocationPrompt;
  geolocation: MatchMode;
  sound: boolean;
  images: boolean;
  video: boolean;
  windowSize: WindowSizeMode;
  windowWidth?: number;
  windowHeight?: number;
  windowPosition: PositionQuadrant;
}

/**
 * 高级指纹设置
 */
export interface AdvancedFingerprintSettings {
  resolution: Resolution;
  colorDepth: 24 | 32; // 屏幕深度
  devicePixelRatio: number; // 像素比
  maxTouchPoints: number; // 最大触摸点数
  fontFingerprint: FingerprintMode;
  fontList: FontListConfig; // 字体列表配置
  webrtc: WebRTCMode;
  webglImage: FingerprintMode;
  webglInfo: FingerprintMode;
  webglVendor: string;
  webglRenderer: string;
  webgpu: 'webgl-match' | 'real' | 'disable';
  canvas: FingerprintMode;
  audioContext: FingerprintMode;
  speechVoices: FingerprintMode;
  doNotTrack: boolean;
  clientRects: FingerprintMode;
  mediaDevices: FingerprintMode;
}

/**
 * 设备设置
 */
export interface DeviceSettings {
  deviceName: string;
  deviceNameRandom: boolean;
  macAddress: string;
  macAddressMode: 'real' | 'custom';
  hardwareConcurrency: number;
  deviceMemory: number;
  sslFingerprint: boolean;
  portScanProtection: boolean;
  scanWhitelist: string;
  hardwareAcceleration: boolean;
  disableSandbox: boolean;
  startupParameters: string;
}

/**
 * 偏好设置
 */
export interface PreferenceSettings {
  syncWindowName: boolean;
  customBookmarks: boolean;
  syncBookmarks: boolean;
  syncHistory: boolean;
  syncTabs: boolean;
  syncCookies: boolean;
  syncExtensions: boolean;
  syncSavedPasswords: boolean;
  syncIndexedDB: boolean;
  syncLocalStorage: boolean;
  deleteCacheBeforeLaunch: boolean;
  deleteCookiesBeforeLaunch: boolean;
  deleteLocalStorageBeforeLaunch: boolean;
  randomFingerprintOnLaunch: boolean;
  showSavePasswordPrompt: boolean;
  stopOpenIfNetworkUnavailable: boolean;
  stopOpenIfIpChanges: boolean;
  stopOpenIfCountryChanges: boolean;
  openWorkbench: boolean | 'follow-software';
  ipChangeReminder: boolean | 'follow-software';
  enableGoogleLogin: boolean | 'follow-software';
  urlBlacklist: string[];
  urlWhitelist: string[];
}

/**
 * 项目元数据
 */
export interface ProjectMetadata {
  defaultProject: string;
  tags: string[];
}

/**
 * 完整的窗口配置
 */
export interface WindowConfig {
  windowInfo: WindowInfo;
  basicSettings: BasicSettings;
  advancedFingerprintSettings: AdvancedFingerprintSettings;
  deviceSettings: DeviceSettings;
  preferenceSettings: PreferenceSettings;
  projectMetadata: ProjectMetadata;
  cookies?: string; // JSON 格式的 Cookies
}

/**
 * 批量创建窗口配置
 */
export interface BatchCreateConfig {
  baseConfig: WindowConfig;
  count: number;
  namePattern: string; // 例如：窗口_{index}
  variations?: {
    proxyIps?: string[];
    userAgent?: string[];
    // 其他需要变化的字段
  };
}

/**
 * 导入窗口数据
 */
export interface ImportWindowData {
  windows: WindowConfig[];
  overwrite?: boolean;
}
