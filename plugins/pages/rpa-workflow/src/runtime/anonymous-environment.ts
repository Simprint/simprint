import { invoke } from '@/lib/tauri';
import {
  type BrowserKernelVersion,
  type ProxyConfig,
  listBrowserKernels,
  SIMPRINT_KERNEL_CHROMIUM,
} from '../../../../services/environment/src';
import { getStorageSettings } from '../../../../services/store/src/stores/storage-settings/storage-settings-store';
import { getEnvironmentCdpEndpoint, type CdpEndpoint } from './tauri';

interface DefaultStoragePaths {
  profiles: string;
  cache: string;
}

interface FingerprintPayload {
  platform?: string;
  env_id?: string;
  env_name?: string;
  window_width?: number;
  window_height?: number;
}

function createAnonymousEnvUuid(): string {
  return `0000-${crypto.randomUUID()}`;
}

function pickRunnableKernel(kernels: BrowserKernelVersion[]): BrowserKernelVersion | null {
  const runnable = kernels.filter(
    (kernel) =>
      kernel.status === 'active' && !!kernel.url && !!kernel.hash && !!kernel.signature?.trim()
  );

  if (runnable.length === 0) {
    return null;
  }

  return runnable.find((kernel) => kernel.is_latest) || runnable[0] || null;
}

function resolvePlatform(): { system: string; platform: string } {
  const ua = navigator.userAgent.toLowerCase();

  if (ua.includes('mac os')) {
    return { system: 'macOS', platform: 'darwin' };
  }

  if (ua.includes('linux')) {
    return { system: 'Linux', platform: 'linux' };
  }

  return { system: 'Windows', platform: 'windows' };
}

async function waitForEndpoint(envUuid: string, timeoutMs = 60000): Promise<CdpEndpoint> {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    const endpoint = await getEnvironmentCdpEndpoint(envUuid);
    if (endpoint?.browser_ws_url) {
      return endpoint;
    }

    await new Promise((resolve) => window.setTimeout(resolve, 500));
  }

  throw new Error('Temporary RPA environment started, but CDP endpoint was not ready in time.');
}

export async function startAnonymousRpaEnvironment(
  proxy?: ProxyConfig | null
): Promise<CdpEndpoint> {
  const [storage, defaultPaths] = await Promise.all([
    getStorageSettings(),
    invoke<DefaultStoragePaths>('get_storage_default_paths'),
  ]);

  const { system, platform } = resolvePlatform();
  const effectiveProfiles =
    (typeof storage.profilesPath === 'string' && storage.profilesPath.trim()) || defaultPaths.profiles;
  const effectiveCache =
    (typeof storage.cachePath === 'string' && storage.cachePath.trim()) || defaultPaths.cache;

  const kernelsMap = await listBrowserKernels(platform, SIMPRINT_KERNEL_CHROMIUM);
  const kernelList = kernelsMap[SIMPRINT_KERNEL_CHROMIUM] || [];
  const kernelDetail = pickRunnableKernel(kernelList);
  if (!kernelDetail) {
    throw new Error(`No runnable browser kernel is available for ${platform}.`);
  }

  const anonymousEnvUuid = createAnonymousEnvUuid();

  const exePath = await invoke<string>('ensure_kernel_ready', {
    envUuid: anonymousEnvUuid,
    kernelValue: kernelDetail.kernel_id,
    profilesPath: effectiveProfiles,
    kernelDetail: {
      url: kernelDetail.url,
      hash: kernelDetail.hash,
      signature: kernelDetail.signature!.trim(),
      requires_extract: !!kernelDetail.requires_extract,
    },
  });

  const fingerprintConfig: FingerprintPayload = {
    platform: system,
    env_id: '0',
    env_name: `Anonymous RPA ${anonymousEnvUuid.slice(0, 13)}`,
    window_width: 1280,
    window_height: 900,
  };

  await invoke('launch_environment', {
    exePath,
    envUuid: anonymousEnvUuid,
    cachePath: effectiveCache,
    proxy: proxy ?? undefined,
    fingerprintConfig,
    accounts: undefined,
    extensions: undefined,
  });

  return waitForEndpoint(anonymousEnvUuid);
}

export async function stopAnonymousRpaEnvironment(envUuid: string): Promise<void> {
  await invoke('stop_environment', { envUuid });
}
