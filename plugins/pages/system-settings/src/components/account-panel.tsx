import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { KeyRound, Lock, Shield, Timer, User } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { SettingCard } from './setting-card';
import { SettingRow } from './setting-row';
import {
  getAccountSecuritySettings,
  setAccountSecuritySettings,
  LOCK_TIME_OPTIONS,
  useAuthStore,
} from '../../../../services/store/src';

/** 本地用户资料与设备安全设置。 */
export const AccountPanel: React.FC = () => {
  const { t } = useTranslation('settings');
  const user = useAuthStore((state) => state.user);
  const [autoLockEnabled, setAutoLockEnabled] = useState(false);
  const [autoLockTime, setAutoLockTime] = useState(5);
  const [cleanDataOnExit, setCleanDataOnExit] = useState(false);
  const [securitySettingsLoaded, setSecuritySettingsLoaded] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void getAccountSecuritySettings().then((settings) => {
      if (!cancelled) {
        setAutoLockEnabled(settings.autoLockEnabled);
        setAutoLockTime(settings.autoLockTime);
        setCleanDataOnExit(settings.cleanDataOnExit);
        setSecuritySettingsLoaded(true);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleAutoLockChange = useCallback((checked: boolean) => {
    setAutoLockEnabled(checked);
    void setAccountSecuritySettings({ autoLockEnabled: checked });
  }, []);

  const handleAutoLockTimeChange = useCallback((value: string) => {
    const minutes = Number(value);
    setAutoLockTime(minutes);
    void setAccountSecuritySettings({ autoLockTime: minutes });
  }, []);

  const handleCleanDataOnExitChange = useCallback((checked: boolean) => {
    setCleanDataOnExit(checked);
    void setAccountSecuritySettings({ cleanDataOnExit: checked });
  }, []);

  return (
    <div className="space-y-6">
      <SettingCard title={t('accountProfile')} icon={User}>
        <div className="flex items-center gap-4 p-3">
          <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-full border-2 border-primary/20 bg-gradient-to-br from-primary/10 to-primary/25 text-3xl">
            {user?.avatar || '🙂'}
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate text-base font-semibold text-foreground">
              {user?.nickname || t('localUser')}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">{t('localAccountDesc')}</p>
          </div>
        </div>
        <div className="mt-2 pt-2">
          <SettingRow
            icon={KeyRound}
            title={t('localPassword')}
            description={user?.has_password ? t('passwordProtected') : t('passwordlessAccount')}
          >
            <span />
          </SettingRow>
        </div>
      </SettingCard>

      <SettingCard title={t('accountSecurity')} icon={Shield}>
        <SettingRow icon={Timer} title={t('autoLock')} description={t('autoLockDesc')}>
          <Switch
            checked={autoLockEnabled}
            onCheckedChange={handleAutoLockChange}
            disabled={!securitySettingsLoaded}
          />
        </SettingRow>
        {autoLockEnabled && (
          <div className="ml-7 flex items-center justify-between rounded-lg bg-accent/20 p-3">
            <span className="text-sm text-foreground">{t('lockTime')}</span>
            <Select
              value={String(autoLockTime)}
              onValueChange={handleAutoLockTimeChange}
              disabled={!securitySettingsLoaded}
            >
              <SelectTrigger className="w-24">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LOCK_TIME_OPTIONS.map((minutes) => (
                  <SelectItem key={minutes} value={String(minutes)}>
                    {t('minutes', { count: minutes })}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}
        <SettingRow icon={Lock} title={t('exitClearData')} description={t('exitClearDataDesc')}>
          <Switch
            checked={cleanDataOnExit}
            onCheckedChange={handleCleanDataOnExitChange}
            disabled={!securitySettingsLoaded}
          />
        </SettingRow>
      </SettingCard>
    </div>
  );
};
