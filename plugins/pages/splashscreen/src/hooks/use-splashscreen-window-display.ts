import { useEffect, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@/lib/tauri';

interface UseSplashscreenWindowDisplayReturn {
  contentRef: React.RefObject<HTMLDivElement | null>;
}

/**
 * Splashscreen 窗口显示控制 Hook
 * 负责在内容准备好后显示窗口并通知后端
 */
export function useSplashscreenWindowDisplay(): UseSplashscreenWindowDisplayReturn {
  const contentRef = useRef<HTMLDivElement>(null);
  const hasShownWindow = useRef(false);

  useEffect(() => {
    let animationFrameId: number;

    const checkContentReady = () => {
      // 检查 DOM 元素是否已存在且可见（有高度）
      const isContentReady =
        contentRef.current &&
        contentRef.current.offsetHeight > 0 &&
        contentRef.current.offsetWidth > 0;

      if (isContentReady && !hasShownWindow.current) {
        // 内容已渲染，使用双重 RAF 确保浏览器完成首帧布局后再显示窗口。
        hasShownWindow.current = true;

        requestAnimationFrame(() => {
          requestAnimationFrame(async () => {
            try {
              const splashWindow = getCurrentWindow();

              // 检查窗口标签是否匹配
              if (splashWindow.label.includes('splashscreen')) {
                await splashWindow.show();
                await splashWindow.setFocus();
                // 窗口显示后，通知后端开始加载
                try {
                  await invoke('splashscreen_ready');
                } catch (error) {
                  console.error('[SplashscreenWindowDisplay] 通知后端开始加载失败:', error);
                }
              }
            } catch (error) {
              console.error('[SplashscreenWindowDisplay] 显示窗口失败:', error);
            }
          });
        });
      } else if (!isContentReady) {
        // 内容还未准备好，继续检查（使用 RAF 避免阻塞）
        animationFrameId = requestAnimationFrame(checkContentReady);
      }
    };

    // 立即开始检查，使用 RAF 循环直到内容准备好
    animationFrameId = requestAnimationFrame(checkContentReady);

    return () => {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  }, []);

  return {
    contentRef,
  };
}
