import fs from 'node:fs';
import path from 'node:path';

const CONFIG_BY_MODE = {
  embedBootstrapper: 'tauri.conf.embed-bootstrapper.json',
  'fixed-runtime': 'tauri.conf.fixed-runtime.json',
};

const TARGET_BY_NODE_ARCH = {
  x64: 'x86_64-pc-windows-msvc',
  arm64: 'aarch64-pc-windows-msvc',
  ia32: 'i686-pc-windows-msvc',
};

const FIXED_RUNTIME_PATH_BY_TARGET = {
  'x86_64-pc-windows-msvc':
    './webview-fixed/Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.x64/',
  'aarch64-pc-windows-msvc':
    './webview-fixed/Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.arm64/',
  'i686-pc-windows-msvc':
    './webview-fixed/Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.x86/',
};

const mode = process.argv[2];
const sourceName = CONFIG_BY_MODE[mode];
const target = process.argv[3] || TARGET_BY_NODE_ARCH[process.arch];

if (!sourceName) {
  throw new Error(
    `[prepare-tauri-config] Unsupported mode "${mode ?? ''}". ` +
      'Expected embedBootstrapper or fixed-runtime.'
  );
}

if (!FIXED_RUNTIME_PATH_BY_TARGET[target]) {
  throw new Error(
    `[prepare-tauri-config] Unsupported Windows target "${target ?? ''}". ` +
      'Expected x86_64-pc-windows-msvc, aarch64-pc-windows-msvc or i686-pc-windows-msvc.'
  );
}

const root = path.resolve(process.cwd());
const sourcePath = path.join(root, 'src-tauri', sourceName);
const targetPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const config = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));
const publicKey = process.env.TAURI_UPDATER_PUBLIC_KEY?.trim();

if (mode === 'fixed-runtime') {
  config.bundle.windows.webviewInstallMode.path = FIXED_RUNTIME_PATH_BY_TARGET[target];
}

if (publicKey) {
  config.plugins ??= {};
  config.plugins.updater ??= {};
  config.plugins.updater.pubkey = publicKey;
}

fs.writeFileSync(targetPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8');
console.log(`[prepare-tauri-config] Prepared ${mode} for ${target} from src-tauri/${sourceName}`);
