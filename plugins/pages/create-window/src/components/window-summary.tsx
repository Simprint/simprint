import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Shuffle, Fingerprint } from 'lucide-react';
import type { WindowConfig } from '../types';

interface WindowSummaryProps {
  config: WindowConfig;
  onRandomize?: () => void;
}

// 概要项组件
function SummaryItem({
  label,
  value,
  mono,
  wrap,
}: {
  label: string;
  value: string;
  mono?: boolean;
  wrap?: boolean;
}) {
  return (
    <div className={`flex justify-between gap-2 ${wrap ? 'items-start' : 'items-center'}`}>
      <span className="text-muted-foreground shrink-0">{label}</span>
      <span
        className={`flex-1 min-w-0 ${wrap ? 'text-right break-all whitespace-normal' : 'truncate text-right'} ${mono ? 'font-mono' : ''}`}
        title={value}
      >
        {value}
      </span>
    </div>
  );
}

// 概要分组组件
function SummaryGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <h4 className="text-xs font-medium text-muted-foreground">{title}</h4>
      <div className="space-y-1.5 text-xs">{children}</div>
    </div>
  );
}

export function WindowSummary({ config, onRandomize }: WindowSummaryProps) {
  const { t } = useTranslation('create-window');

  return (
    <div className="p-4 space-y-4">
      {/* 概要 */}
      <div className="relative space-y-4 p-4 border border-border rounded-lg overflow-hidden">
        {/* 背景装饰 */}
        <Fingerprint className="absolute -bottom-12 -right-12 w-48 h-48 text-muted-foreground/5 pointer-events-none" />

        <div className="flex items-center justify-between relative z-10">
          <h3 className="text-sm font-semibold">{t('summary.title')}</h3>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={onRandomize}
            title={t('summary.randomFingerprint')}
          >
            <Shuffle className="h-3.5 w-3.5" />
          </Button>
        </div>

        {/* 基本信息 */}
        <SummaryGroup title={t('sections.basicInfo')}>
          <SummaryItem label={t('windowInfo.name')} value={config.windowInfo.name || '-'} />
          <SummaryItem label={t('windowInfo.system')} value={config.windowInfo.system} />
          <SummaryItem
            label={t('windowInfo.userAgent')}
            value={config.windowInfo.userAgent || '-'}
            mono
            wrap
          />
          <SummaryItem
            label={t('windowInfo.platformAccount')}
            value={
              config.windowInfo.accountUuids && config.windowInfo.accountUuids.length > 0
                ? `${config.windowInfo.accountUuids.length} ${t('summary.accountsLinked')}`
                : t('windowInfo.noPlatformAccount')
            }
          />
        </SummaryGroup>

        {/* 网络与定位 */}
        <SummaryGroup title={t('sections.networkLocation')}>
          <SummaryItem
            label={t('windowInfo.proxyIp')}
            value={
              (config.windowInfo.proxySourceMode === 'local'
                ? config.windowInfo.localProxyNodeNames.length
                : config.windowInfo.proxyUuids.length) > 0
                ? `${
                    config.windowInfo.proxySourceMode === 'local'
                      ? config.windowInfo.localProxyNodeNames.length
                      : config.windowInfo.proxyUuids.length
                  } ${t('summary.proxiesConfigured')}`
                : t('windowInfo.noProxy')
            }
          />
          <SummaryItem
            label={t('advancedFingerprint.webrtc')}
            value={t(`advancedFingerprint.webrtcMode.${config.advancedFingerprintSettings.webrtc}`)}
          />
          <SummaryItem
            label={t('summary.language')}
            value={
              config.basicSettings.language === 'ip'
                ? t('basicSettings.matchMode.ip')
                : t('basicSettings.matchMode.custom')
            }
          />
          <SummaryItem
            label={t('summary.timezone')}
            value={
              config.basicSettings.timezone === 'ip'
                ? t('basicSettings.matchMode.ip')
                : t('basicSettings.matchMode.custom')
            }
          />
          <SummaryItem
            label={t('summary.geolocation')}
            value={
              config.basicSettings.geolocation === 'ip'
                ? t('basicSettings.matchMode.ip')
                : t('basicSettings.matchMode.custom')
            }
          />
        </SummaryGroup>

        {/* 指纹伪装 */}
        <SummaryGroup title={t('sections.fingerprint')}>
          <SummaryItem
            label={t('advancedFingerprint.canvas')}
            value={t(
              `advancedFingerprint.fingerprintMode.${config.advancedFingerprintSettings.canvas}`
            )}
          />
          <SummaryItem
            label={t('advancedFingerprint.audioContext')}
            value={t(
              `advancedFingerprint.fingerprintMode.${config.advancedFingerprintSettings.audioContext}`
            )}
          />
          <SummaryItem
            label={t('advancedFingerprint.webglImage')}
            value={t(
              `advancedFingerprint.fingerprintMode.${config.advancedFingerprintSettings.webglImage}`
            )}
          />
          <SummaryItem
            label={t('advancedFingerprint.fontFingerprint')}
            value={t(
              `advancedFingerprint.fingerprintMode.${config.advancedFingerprintSettings.fontFingerprint}`
            )}
          />
        </SummaryGroup>

        {/* 屏幕与硬件 */}
        <SummaryGroup title={t('sections.screenHardware')}>
          <SummaryItem
            label={t('summary.resolution')}
            value={`${config.advancedFingerprintSettings.resolution.width} × ${config.advancedFingerprintSettings.resolution.height}`}
          />
          <SummaryItem
            label={t('advancedFingerprint.colorDepth')}
            value={`${config.advancedFingerprintSettings.colorDepth}bit`}
          />
          <SummaryItem
            label={t('advancedFingerprint.devicePixelRatio')}
            value={config.advancedFingerprintSettings.devicePixelRatio.toString()}
          />
          <SummaryItem
            label={t('deviceSettings.hardwareConcurrency')}
            value={`${config.deviceSettings.hardwareConcurrency}${t('deviceSettings.cores')}`}
          />
          <SummaryItem
            label={t('deviceSettings.deviceMemory')}
            value={`${config.deviceSettings.deviceMemory}G`}
          />
        </SummaryGroup>
      </div>
    </div>
  );
}
