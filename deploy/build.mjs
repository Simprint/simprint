import { spawnSync } from 'node:child_process';

const WEBVIEW_MODES = new Set(['embedBootstrapper', 'fixed-runtime']);
const webviewMode = process.argv[2] || 'embedBootstrapper';
const target = process.argv[3];

if (!WEBVIEW_MODES.has(webviewMode)) {
  console.error(
    `[deploy/build] Unsupported WebView mode "${webviewMode}". Expected embedBootstrapper or fixed-runtime.`
  );
  process.exit(1);
}

function run(cmd, args, extraEnv = {}) {
  const res = spawnSync(cmd, args, {
    stdio: 'inherit',
    shell: process.platform === 'win32',
    env: { ...process.env, ...extraEnv },
  });
  if (res.status !== 0) {
    process.exit(res.status ?? 1);
  }
}

// 1) Select the complete Tauri config, then sync the release version into the working config.
run('node', ['deploy/prepare-tauri-config.mjs', webviewMode, ...(target ? [target] : [])]);
run('node', ['deploy/prepare-version.mjs']);

// 2) Build frontend (tauri.conf.json uses beforeBuildCommand, but we keep this explicit for local usage)
run('node', ['build.cjs']);

// 3) Tauri build (local usage; CI uses tauri-action)
const tauriArgs = ['-s', 'exec', 'tauri', 'build', '--features', 'production'];
if (target) {
  tauriArgs.push('--target', target);
}
run('pnpm', tauriArgs, { SIMPRINT_WEBVIEW_MODE: webviewMode, ENV_NAME: 'production' });
