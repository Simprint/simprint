import { useSplashscreenInit } from '../hooks/useSplashscreenInit';
import { useSplashscreenEvents } from '../hooks/useSplashscreenEvents';
import { useSplashscreenWindowDisplay } from '../hooks/use-splashscreen-window-display';
import { useSplashscreenClose } from '../hooks/use-splashscreen-close';
import {
  SplashscreenBackground,
  SplashscreenLogo,
  SplashscreenContent,
  SplashscreenLoadingText,
  SplashscreenErrorMessage,
  SplashscreenDecoration,
} from '.';
import { SplashscreenCloseButton } from './splashscreen-close-button';

/** Splashscreen 页面，只依赖启动窗口所需的最小组件集合。 */
export const SplashscreenPage: React.FC = () => {
  useSplashscreenInit();

  const { loadingText, errorMessage, connectionFailed, showCloseButton, progress, isUpdating } =
    useSplashscreenEvents();
  const { contentRef } = useSplashscreenWindowDisplay();
  const { handleClose } = useSplashscreenClose();

  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-50/80 via-blue-100/60 to-blue-50/80 flex items-center justify-center overflow-hidden">
      <SplashscreenBackground />
      {showCloseButton && <SplashscreenCloseButton onClose={handleClose} />}

      <div ref={contentRef} className="relative z-10 flex flex-col items-center gap-12">
        <SplashscreenLogo />
        <SplashscreenContent />
        {errorMessage && !connectionFailed && <SplashscreenErrorMessage message={errorMessage} />}
      </div>

      <SplashscreenLoadingText
        loadingText={loadingText}
        progress={progress}
        isUpdating={isUpdating}
      />
      <SplashscreenDecoration />
    </div>
  );
};
