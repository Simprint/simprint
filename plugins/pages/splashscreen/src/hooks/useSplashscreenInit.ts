import { useEffect } from 'react';
import { invoke } from '@/lib/tauri';

/**
 * Splashscreen 初始化 Hook
 * 负责检查应用状态
 */
export function useSplashscreenInit() {
  useEffect(() => {
    const init = async () => {
      try {
        const appState = (await invoke('get_app_state')) as { is_initialized?: boolean };

        if (appState?.is_initialized) {
          await invoke('show_main_window');
        }
      } catch (error) {
        console.error('[SplashscreenInit] Initialization failed:', error);
      }
    };

    init();
  }, []);
}
