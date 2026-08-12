import { useEffect, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@/lib/tauri';
import { elapsedSinceMainHtml, mainStartupTiming } from '@/main-startup-timing';

/** 在关键应用状态和首帧布局全部就绪后，向后端报告隐藏主窗口已就绪。 */
export const WindowManagerService: React.FC = () => {
  const hasReportedReady = useRef(false);

  useEffect(() => {
    let animationFrameId: number;
    let checkCount = 0;
    mainStartupTiming.windowManagerEffectStartedAt = performance.now();

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
        mainStartupTiming.firstContentLayoutAt = performance.now();

        try {
          await document.fonts.ready;
          mainStartupTiming.fontsReadyAt = performance.now();
          requestAnimationFrame(() => {
            requestAnimationFrame(async () => {
              try {
                mainStartupTiming.readyInvokeAt = performance.now();
                const frontendTiming = {
                  navigationStartEpochMs: performance.timeOrigin,
                  navigationToHtmlMs: mainStartupTiming.htmlStartedAt,
                  htmlToModuleMs: elapsedSinceMainHtml(mainStartupTiming.moduleExecutedAt),
                  htmlToReactRenderMs: elapsedSinceMainHtml(
                    mainStartupTiming.reactRenderScheduledAt
                  ),
                  htmlToWindowManagerEffectMs: elapsedSinceMainHtml(
                    mainStartupTiming.windowManagerEffectStartedAt
                  ),
                  htmlToFirstContentLayoutMs: elapsedSinceMainHtml(
                    mainStartupTiming.firstContentLayoutAt
                  ),
                  htmlToFontsReadyMs: elapsedSinceMainHtml(mainStartupTiming.fontsReadyAt),
                  htmlToReadyInvokeMs: elapsedSinceMainHtml(mainStartupTiming.readyInvokeAt),
                };

                void invoke('log_info', {
                  module: 'simprint::frontend::main',
                  message: `Main window frontend timing: ${JSON.stringify(frontendTiming)}`,
                }).catch((error) => {
                  console.warn('[WindowManagerService] failed to report startup timing:', error);
                });

                await invoke('main_window_ready');
                window.dispatchEvent(new Event('simprint:main-window-ready'));
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
