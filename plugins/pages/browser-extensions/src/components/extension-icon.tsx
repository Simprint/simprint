import { useMemo, useState } from 'react';
import { Puzzle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { resolveExtensionIconSrc } from '../utils/icon';

interface ExtensionIconProps {
  icon?: string | null;
  source?: 'local';
  containerClassName?: string;
  imageClassName?: string;
  textClassName?: string;
  fallbackClassName?: string;
  allowTextFallback?: boolean;
}

function isLikelyIconText(icon: string): boolean {
  const trimmed = icon.trim();
  if (!trimmed || trimmed.length > 4) {
    return false;
  }

  return !/[\\/:.]/.test(trimmed);
}

export function ExtensionIcon({
  icon,
  source = 'local',
  containerClassName,
  imageClassName,
  textClassName,
  fallbackClassName,
  allowTextFallback = true,
}: ExtensionIconProps) {
  const resolvedIconSrc = useMemo(() => resolveExtensionIconSrc(icon), [icon]);
  const [failedIconSrc, setFailedIconSrc] = useState<string | null>(null);

  const iconSrc = useMemo(() => {
    if (!resolvedIconSrc || failedIconSrc === resolvedIconSrc) {
      return undefined;
    }
    return resolvedIconSrc;
  }, [failedIconSrc, resolvedIconSrc]);

  return (
    <div
      className={cn(
        'flex items-center justify-center overflow-hidden shrink-0',
        containerClassName
      )}
    >
      {iconSrc ? (
        <img
          src={iconSrc}
          alt=""
          className={cn('h-full w-full object-cover', imageClassName)}
          onError={() => setFailedIconSrc(iconSrc)}
        />
      ) : icon && allowTextFallback && isLikelyIconText(icon) ? (
        <span className={cn('text-2xl leading-none', textClassName)}>{icon}</span>
      ) : (
        <Puzzle className={cn('text-sky-600/70', fallbackClassName)} />
      )}
    </div>
  );
}
