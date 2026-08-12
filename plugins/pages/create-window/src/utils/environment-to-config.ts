/**
 * 将环境详情转换为窗口配置
 */
import type { WindowConfig } from '../types';
import type {
  EnvironmentDetailResponse,
  EnvironmentConfigRequest,
} from '../../../environment-manager/src/api';
import { generateWebGL } from './fingerprint-generator';
import { normalizeFontListConfig, normalizeResolution } from './config-normalizers';

/**
 * 将后端环境配置转换为前端 WindowConfig
 */
export function transformEnvironmentConfigToWindowConfig(
  detail: EnvironmentDetailResponse
): WindowConfig {
  const { environment, config } = detail;
  const envConfig = config || ({} as EnvironmentConfigRequest);

  // 提取配置数据，使用类型断言和默认值
  const windowInfo = (envConfig.window_info || {}) as Record<string, unknown>;
  const basicSettings = (envConfig.basic_settings || {}) as Record<string, unknown>;
  const fingerprintSettings = (envConfig.fingerprint_settings || {}) as Record<string, unknown>;
  const deviceSettings = (envConfig.device_settings || {}) as Record<string, unknown>;
  const preferenceSettings = (envConfig.preference_settings || {}) as Record<string, unknown>;
  const projectMetadata = (envConfig.project_metadata || {}) as Record<string, unknown>;

  // 从 detail 中获取代理和分组信息（现在后端直接返回完整对象）
  const proxyUuids = detail.proxy?.uuid ? [detail.proxy.uuid] : [];
  const cookies = (detail.cookies || [])
    .map((item) => ({
      site: item.site,
      cookieText: item.cookie_text,
    }))
    .filter((item) => item.site.length > 0 && item.cookieText.length > 0);
  const urls = (detail.urls || []).map((item) => item.url).filter(Boolean);

  // 生成关联的 WebGL 默认值
  const webglPair = generateWebGL();
  const system = (windowInfo.system as string) || 'Windows';
  const fontList = normalizeFontListConfig(fingerprintSettings.fontList, system);
  const resolution = normalizeResolution(fingerprintSettings.resolution);

  return {
    windowInfo: {
      name: (windowInfo.name as string) || environment.name || '',
      system,
      kernel: (windowInfo.kernel as string) || 'Chrome',
      kernelId: (windowInfo.kernel_id as string) || '',
      userAgent: (windowInfo.userAgent as string) || '',
      searchEngine: (windowInfo.searchEngine as string) || 'Google',
      proxySourceMode: 'remote',
      proxyUuids, // 代理 UUID 从 detail.proxy 中获取，转换为数组
      localProxyNodeNames: [],
      accountUuids: (detail.accounts || []).map((acc) => acc.uuid), // 账号 UUID 列表从 detail.accounts 中获取
      urls,
      cookies,
      description: (windowInfo.description as string) || environment.description || '',
    },
    basicSettings: {
      language: (basicSettings.language as 'ip' | 'custom') || 'ip',
      interfaceLanguage: (basicSettings.interfaceLanguage as 'ip' | 'custom') || 'ip',
      timezone: (basicSettings.timezone as 'ip' | 'custom') || 'ip',
      geolocationPrompt: (basicSettings.geolocationPrompt as 'ask' | 'allow' | 'forbid') || 'allow',
      geolocation: (basicSettings.geolocation as 'ip' | 'custom') || 'ip',
      sound: (basicSettings.sound as boolean) ?? true,
      images: (basicSettings.images as boolean) ?? true,
      video: (basicSettings.video as boolean) ?? true,
      windowSize: (basicSettings.windowSize as 'custom' | 'fullscreen') || 'custom',
      windowWidth: (basicSettings.windowWidth as number) || 1000,
      windowHeight: (basicSettings.windowHeight as number) || 1000,
      windowPosition:
        (basicSettings.windowPosition as
          | 'top-left'
          | 'top-right'
          | 'bottom-left'
          | 'bottom-right') || 'top-left',
    },
    advancedFingerprintSettings: {
      resolution,
      colorDepth: (fingerprintSettings.colorDepth as 24 | 32) || 24,
      devicePixelRatio: (fingerprintSettings.devicePixelRatio as number) || 1.0,
      maxTouchPoints: (fingerprintSettings.maxTouchPoints as number) || 0,
      fontFingerprint:
        (fingerprintSettings.fontFingerprint as 'random' | 'real' | 'custom' | 'system') ||
        'system',
      fontList,
      webrtc: (fingerprintSettings.webrtc as 'replace' | 'real' | 'disable') || 'disable',
      webglImage:
        (fingerprintSettings.webglImage as 'random' | 'real' | 'custom' | 'system') || 'random',
      webglInfo:
        (fingerprintSettings.webglInfo as 'random' | 'real' | 'custom' | 'system') || 'random',
      webglVendor: (fingerprintSettings.webglVendor as string) || webglPair.vendor,
      webglRenderer: (fingerprintSettings.webglRenderer as string) || webglPair.renderer,
      webgpu: (fingerprintSettings.webgpu as 'webgl-match' | 'real' | 'disable') || 'webgl-match',
      canvas: (fingerprintSettings.canvas as 'random' | 'real' | 'custom' | 'system') || 'random',
      audioContext:
        (fingerprintSettings.audioContext as 'random' | 'real' | 'custom' | 'system') || 'random',
      speechVoices:
        (fingerprintSettings.speechVoices as 'random' | 'real' | 'custom' | 'system') || 'random',
      doNotTrack: (fingerprintSettings.doNotTrack as boolean) ?? true,
      clientRects:
        (fingerprintSettings.clientRects as 'random' | 'real' | 'custom' | 'system') || 'random',
      mediaDevices:
        (fingerprintSettings.mediaDevices as 'random' | 'real' | 'custom' | 'system') || 'random',
    },
    deviceSettings: {
      deviceName: (deviceSettings.deviceName as string) || '',
      deviceNameRandom: (deviceSettings.deviceNameRandom as boolean) ?? false,
      macAddress: (deviceSettings.macAddress as string) || '',
      macAddressMode: (deviceSettings.macAddressMode as 'real' | 'custom') || 'real',
      hardwareConcurrency: (deviceSettings.hardwareConcurrency as number) || 4,
      deviceMemory: (deviceSettings.deviceMemory as number) || 8,
      sslFingerprint: (deviceSettings.sslFingerprint as boolean) ?? false,
      portScanProtection: (deviceSettings.portScanProtection as boolean) ?? false,
      scanWhitelist: (deviceSettings.scanWhitelist as string) || '',
      hardwareAcceleration: (deviceSettings.hardwareAcceleration as boolean) ?? true,
      disableSandbox: (deviceSettings.disableSandbox as boolean) ?? false,
      startupParameters: (deviceSettings.startupParameters as string) || '',
    },
    preferenceSettings: {
      syncWindowName: (preferenceSettings.syncWindowName as boolean) ?? false,
      customBookmarks: (preferenceSettings.customBookmarks as boolean) ?? false,
      syncBookmarks: (preferenceSettings.syncBookmarks as boolean) ?? false,
      syncHistory: (preferenceSettings.syncHistory as boolean) ?? false,
      syncTabs: (preferenceSettings.syncTabs as boolean) ?? false,
      syncCookies: (preferenceSettings.syncCookies as boolean) ?? false,
      syncExtensions: (preferenceSettings.syncExtensions as boolean) ?? false,
      syncSavedPasswords: (preferenceSettings.syncSavedPasswords as boolean) ?? false,
      syncIndexedDB: (preferenceSettings.syncIndexedDB as boolean) ?? false,
      syncLocalStorage: (preferenceSettings.syncLocalStorage as boolean) ?? false,
      deleteCacheBeforeLaunch: (preferenceSettings.deleteCacheBeforeLaunch as boolean) ?? false,
      deleteCookiesBeforeLaunch: (preferenceSettings.deleteCookiesBeforeLaunch as boolean) ?? false,
      deleteLocalStorageBeforeLaunch:
        (preferenceSettings.deleteLocalStorageBeforeLaunch as boolean) ?? false,
      randomFingerprintOnLaunch: (preferenceSettings.randomFingerprintOnLaunch as boolean) ?? false,
      showSavePasswordPrompt: (preferenceSettings.showSavePasswordPrompt as boolean) ?? false,
      stopOpenIfNetworkUnavailable:
        (preferenceSettings.stopOpenIfNetworkUnavailable as boolean) ?? false,
      stopOpenIfIpChanges: (preferenceSettings.stopOpenIfIpChanges as boolean) ?? false,
      stopOpenIfCountryChanges: (preferenceSettings.stopOpenIfCountryChanges as boolean) ?? false,
      openWorkbench: (preferenceSettings.openWorkbench as boolean | 'follow-software') ?? false,
      ipChangeReminder:
        (preferenceSettings.ipChangeReminder as boolean | 'follow-software') ?? false,
      enableGoogleLogin:
        (preferenceSettings.enableGoogleLogin as boolean | 'follow-software') ?? false,
      urlBlacklist: (preferenceSettings.urlBlacklist as string[]) || [],
      urlWhitelist: (preferenceSettings.urlWhitelist as string[]) || [],
    },
    projectMetadata: {
      defaultProject: (projectMetadata.defaultProject as string) || '',
      tags: (projectMetadata.tags as string[]) || [],
    },
  };
}
