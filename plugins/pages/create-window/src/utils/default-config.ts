import type { WindowConfig } from '../types';
import { generateUserAgentByKernel } from './user-agent-generator';
import {
  generateRandomColorDepth,
  generateRandomDevicePixelRatio,
  generateRandomMaxTouchPoints,
  generateRandomHardwareConcurrency,
  generateRandomDeviceMemory,
  generateDeviceName,
  generateMacAddress,
  generateWebGL,
  generateScreenResolution,
  generateFontListConfig,
} from './fingerprint-generator';

/**
 * 获取默认窗口配置
 */
export function getDefaultWindowConfig(): WindowConfig {
  const defaultSystem = 'Windows';
  const defaultKernel = 'Chrome';
  // 不传递版本号，让 UA 生成器自动随机生成
  // 这样每次创建新窗口时都会生成不同的 UA

  // 生成关联的 WebGL vendor 和 renderer
  const webglPair = generateWebGL();

  return {
    windowInfo: {
      name: '',
      system: defaultSystem,
      kernel: defaultKernel,
      kernelId: '',
      userAgent: generateUserAgentByKernel(defaultSystem, defaultKernel),
      searchEngine: 'Google',
      proxySourceMode: 'remote',
      proxyUuids: [],
      localProxyNodeNames: [],
      accountUuids: [],
      urls: [],
      cookies: [],
      description: '',
    },
    basicSettings: {
      language: 'ip',
      interfaceLanguage: 'ip',
      timezone: 'ip',
      geolocationPrompt: 'allow',
      geolocation: 'ip',
      sound: true,
      images: true,
      video: true,
      windowSize: 'custom',
      windowWidth: 1000,
      windowHeight: 1000,
      windowPosition: 'top-left',
    },
    advancedFingerprintSettings: {
      resolution: generateScreenResolution(),
      colorDepth: generateRandomColorDepth(),
      devicePixelRatio: generateRandomDevicePixelRatio(),
      maxTouchPoints: generateRandomMaxTouchPoints(),
      fontFingerprint: 'random',
      fontList: generateFontListConfig('random', defaultSystem),
      webrtc: 'disable',
      webglImage: 'random',
      webglInfo: 'random',
      webglVendor: webglPair.vendor,
      webglRenderer: webglPair.renderer,
      webgpu: 'webgl-match',
      canvas: 'random',
      audioContext: 'random',
      speechVoices: 'random',
      doNotTrack: true,
      clientRects: 'random',
      mediaDevices: 'random',
    },
    deviceSettings: {
      deviceName: generateDeviceName(),
      deviceNameRandom: true,
      macAddress: generateMacAddress(),
      macAddressMode: 'custom',
      hardwareConcurrency: generateRandomHardwareConcurrency(),
      deviceMemory: generateRandomDeviceMemory(),
      sslFingerprint: true,
      portScanProtection: true,
      scanWhitelist: '',
      hardwareAcceleration: true,
      disableSandbox: false,
      startupParameters: '',
    },
    preferenceSettings: {
      syncWindowName: false,
      customBookmarks: false,
      syncBookmarks: false,
      syncHistory: false,
      syncTabs: false,
      syncCookies: false,
      syncExtensions: false,
      syncSavedPasswords: false,
      syncIndexedDB: false,
      syncLocalStorage: false,
      deleteCacheBeforeLaunch: false,
      deleteCookiesBeforeLaunch: false,
      deleteLocalStorageBeforeLaunch: false,
      randomFingerprintOnLaunch: true,
      showSavePasswordPrompt: true,
      stopOpenIfNetworkUnavailable: true,
      stopOpenIfIpChanges: true,
      stopOpenIfCountryChanges: true,
      openWorkbench: true,
      ipChangeReminder: true,
      enableGoogleLogin: true,
      urlBlacklist: [],
      urlWhitelist: [],
    },
    projectMetadata: {
      defaultProject: '',
      tags: [],
    },
  };
}
