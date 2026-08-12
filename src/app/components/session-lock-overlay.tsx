import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock, Loader2 } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useAuthStore } from '../../../plugins/services/store/src';

interface SessionLockOverlayProps {
  open: boolean;
  unlocking: boolean;
  error: string | null;
  onUnlock: (password: string) => Promise<void>;
}

export const SessionLockOverlay: React.FC<SessionLockOverlayProps> = ({
  open,
  unlocking,
  error,
  onUnlock,
}) => {
  const { t } = useTranslation('settings');
  const user = useAuthStore((state) => state.user);
  const [password, setPassword] = useState('');
  const [showUnlockForm, setShowUnlockForm] = useState(false);

  useEffect(() => {
    if (!open) {
      setPassword('');
      setShowUnlockForm(false);
    }
  }, [open]);

  if (!open) {
    return null;
  }

  const displayName = user?.nickname || t('sessionLock.defaultUser');
  const hasPassword = user?.has_password ?? false;

  const handleUnlock = async () => {
    await onUnlock(password);
    setPassword('');
  };

  return (
    <div className="fixed inset-0 z-[120] flex items-center justify-center bg-background/80 backdrop-blur-md">
      <div
        className={cn(
          'w-full max-w-md overflow-hidden rounded-xl',
          showUnlockForm ? 'border border-border bg-card' : 'bg-card/0 shadow-none'
        )}
      >
        <div
          className={cn(
            showUnlockForm ? 'px-5 py-4' : 'px-6 py-8',
            showUnlockForm && 'border-b border-border/50 bg-linear-to-r from-blue-500/10 via-indigo-500/10 to-blue-500/10'
          )}
        >
          <div className="flex flex-col items-center text-center">
            <button
              type="button"
              onClick={() => {
                if (hasPassword) {
                  setShowUnlockForm(true);
                } else {
                  void onUnlock('');
                }
              }}
              className={cn(
                'flex items-center justify-center rounded-full text-primary transition-colors hover:bg-primary/10',
                showUnlockForm ? 'h-14 w-14' : 'h-24 w-24'
              )}
              aria-label={t('sessionLock.unlock')}
            >
              <Lock className={cn(showUnlockForm ? 'h-7 w-7' : 'h-12 w-12')} />
            </button>
            <div className={cn(showUnlockForm ? 'mt-2' : 'mt-5')}>
              <h2 className="text-base font-semibold text-foreground">{t('sessionLock.title')}</h2>
              <p className="mt-1 text-xs text-muted-foreground">
                {showUnlockForm ? t('sessionLock.description') : t('sessionLock.prompt')}
              </p>
            </div>
          </div>
        </div>

        {showUnlockForm && (
          <>
            <div className="space-y-4 p-4">
              <div className="rounded-lg border border-border bg-muted/30 px-3 py-2">
                <div className="text-[11px] uppercase tracking-wide text-muted-foreground">
                  {t('sessionLock.currentUser')}
                </div>
                <div className="mt-1 text-sm font-medium text-foreground">{displayName}</div>
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="session-lock-password"
                  className="text-xs font-medium text-foreground"
                >
                  {t('sessionLock.password')}
                </label>
                <Input
                  id="session-lock-password"
                  type="password"
                  value={password}
                  autoFocus
                  onChange={(event) => setPassword(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter' && !unlocking && password.trim()) {
                      void handleUnlock();
                    }
                  }}
                  placeholder={t('sessionLock.passwordPlaceholder')}
                  className="h-9 text-sm"
                />
              </div>

              {error && <div className="text-xs text-destructive">{error}</div>}
            </div>

            <div className="flex justify-end gap-2 border-t border-border/50 bg-muted/30 px-5 py-3">
              <Button
                onClick={() => void handleUnlock()}
                disabled={unlocking || !password.trim()}
                className="h-8 text-xs"
              >
                {unlocking ? (
                  <>
                    <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                    {t('sessionLock.unlocking')}
                  </>
                ) : (
                  t('sessionLock.unlock')
                )}
              </Button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
