import { useEffect, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@/lib/tauri';

/** 在关键应用状态和首帧布局全部就绪后，向后端报告隐藏主窗口已就绪。 */
export const WindowManagerService: React.FC = () => {
  const hasReportedReady = useRef(false);

  useEffect(() => {
    let animationFrameId: number;
    let checkCount = 0;

    const checkAndReportReady = async () => {
      checkCount++;
      if (hasReportedReady.current) return;

      const currentWindow = getCurrentWindow();
      if (currentWindow.label !== 'main') return;

      const rootElement = document.getElementById('root');
      const appElement = rootElement?.querySelector('.app') as HTMLElement | null;
      const isContentReady =
        document.readyState === 'complete' &&
        rootElement &&
        rootElement.children.length > 0 &&
        rootElement.offsetHeight > 0 &&
        rootElement.offsetWidth > 0 &&
        appElement &&
        appElement.children.length > 0 &&
        appElement.offsetHeight > 0;

      if (isContentReady) {
        hasReportedReady.current = true;

        try {
          await document.fonts.ready;
          requestAnimationFrame(() => {
            requestAnimationFrame(async () => {
              try {
                await invoke('main_window_ready');
                console.log(
                  '[WindowManagerService] 主窗口真正就绪，已报告后端（检查次数:',
                  checkCount,
                  '）'
                );
              } catch (error) {
                console.error('[WindowManagerService] 报告主窗口就绪失败:', error);
                hasReportedReady.current = false;
                animationFrameId = requestAnimationFrame(checkAndReportReady);
              }
            });
          });
        } catch (error) {
          console.error('[WindowManagerService] 等待字体加载失败:', error);
          hasReportedReady.current = false;
          animationFrameId = requestAnimationFrame(checkAndReportReady);
        }
        return;
      }

      if (checkCount % 50 === 0) {
        console.log('[WindowManagerService] 等待主窗口就绪…（检查次数:', checkCount, '）');
      }
      animationFrameId = requestAnimationFrame(checkAndReportReady);
    };

    console.log('[WindowManagerService] 开始检查主窗口内容…');
    animationFrameId = requestAnimationFrame(checkAndReportReady);

    return () => {
      if (animationFrameId) cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return null;
};
