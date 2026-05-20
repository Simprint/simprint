import { useTranslation } from 'react-i18next';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { AdvancedFingerprintSettings, DeviceSettings, ResolutionMode } from '../types';
import { generateScreenResolution, getSystemScreenResolution } from '../utils/fingerprint-generator';

interface ScreenHardwareFormProps {
  advancedSettings: AdvancedFingerprintSettings;
  deviceSettings: DeviceSettings;
  onAdvancedSettingsChange: (value: AdvancedFingerprintSettings) => void;
  onDeviceSettingsChange: (value: DeviceSettings) => void;
}

// 生成随机设备名称
function generateRandomDeviceName(): string {
  const prefixes = ['DESKTOP', 'PC', 'WORKSTATION', 'COMPUTER'];
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
  let suffix = '';
  for (let i = 0; i < 8; i++) {
    suffix += chars[Math.floor(Math.random() * chars.length)];
  }
  return `${prefix}-${suffix}`;
}

// 生成随机 MAC 地址
function generateRandomMacAddress(): string {
  const hex = '0123456789ABCDEF';
  const parts: string[] = [];
  for (let i = 0; i < 6; i++) {
    let part = '';
    for (let j = 0; j < 2; j++) {
      part += hex[Math.floor(Math.random() * 16)];
    }
    parts.push(part);
  }
  return parts.join('-');
}

// 分段控制器组件
function SegmentedControl<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="inline-flex rounded-full bg-muted p-0.5">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            'px-2.5 py-1 text-xs rounded-full transition-all duration-200',
            value === option.value
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

// 数值选择器组件
function NumberSelector({
  value,
  options,
  onChange,
  unit,
}: {
  value: number;
  options: number[];
  onChange: (value: number) => void;
  unit: string;
}) {
  return (
    <div className="inline-flex rounded-full bg-muted p-0.5">
      {options.map((option) => (
        <button
          key={option}
          type="button"
          onClick={() => onChange(option)}
          className={cn(
            'px-2 py-1 text-xs rounded-full transition-all duration-200 min-w-[36px]',
            value === option
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'
          )}
        >
          {option}
          {unit}
        </button>
      ))}
    </div>
  );
}

export function ScreenHardwareForm({
  advancedSettings,
  deviceSettings,
  onAdvancedSettingsChange,
  onDeviceSettingsChange,
}: ScreenHardwareFormProps) {
  const { t } = useTranslation('create-window');

  const resolutionOptions = [
    { value: 'random' as const, label: t('advancedFingerprint.fingerprintMode.random') },
    { value: 'system' as const, label: t('advancedFingerprint.fingerprintMode.system') },
  ];

  const deviceNameModeOptions = [
    { value: 'random' as const, label: t('advancedFingerprint.fingerprintMode.random') },
    { value: 'real' as const, label: t('advancedFingerprint.fingerprintMode.real') },
  ];

  const macAddressModeOptions = [
    { value: 'real' as const, label: t('deviceSettings.macAddressMode.real') },
    { value: 'custom' as const, label: t('deviceSettings.macAddressMode.custom') },
  ];

  // 处理分辨率模式切换
  const handleResolutionModeChange = (mode: ResolutionMode) => {
    const newResolution = mode === 'system'
      ? getSystemScreenResolution()
      : generateScreenResolution();
    onAdvancedSettingsChange({ ...advancedSettings, resolution: newResolution });
  };

  return (
    <div className="space-y-4 p-4 border border-border rounded-lg">
      <h3 className="text-sm font-semibold">{t('sections.screenHardware')}</h3>

      {/* 屏幕设置 */}
      <div className="grid grid-cols-2 gap-x-12 gap-y-4">
        <div className="space-y-1">
          <Label className="text-xs">{t('advancedFingerprint.resolution')}</Label>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{advancedSettings.resolution.width} × {advancedSettings.resolution.height}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() => handleResolutionModeChange('random')}
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
        <div className="space-y-1">
          <Label className="text-xs">{t('advancedFingerprint.colorDepth')}</Label>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{advancedSettings.colorDepth}bit</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() => {
                const newDepth = advancedSettings.colorDepth === 24 ? 32 : 24;
                onAdvancedSettingsChange({ ...advancedSettings, colorDepth: newDepth as 24 | 32 });
              }}
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>

      {/* 像素比 & 触摸点 */}
      <div className="grid grid-cols-2 gap-x-12 gap-y-4">
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('advancedFingerprint.devicePixelRatio')}</Label>
          <SegmentedControl
            value={advancedSettings.devicePixelRatio.toString()}
            options={[
              { value: '1', label: '1.0' },
              { value: '1.5', label: '1.5' },
              { value: '2', label: '2.0' },
            ]}
            onChange={(v) =>
              onAdvancedSettingsChange({ ...advancedSettings, devicePixelRatio: parseFloat(v) })
            }
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('advancedFingerprint.maxTouchPoints')}</Label>
          <SegmentedControl
            value={advancedSettings.maxTouchPoints.toString()}
            options={[
              { value: '0', label: '0' },
              { value: '5', label: '5' },
              { value: '10', label: '10' },
            ]}
            onChange={(v) =>
              onAdvancedSettingsChange({ ...advancedSettings, maxTouchPoints: parseInt(v) })
            }
          />
        </div>
      </div>

      {/* 硬件配置分隔线 */}
      <div className="pt-2 border-t border-border space-y-4">
        {/* CPU & 内存 */}
        <div className="grid grid-cols-2 gap-x-12 gap-y-4">
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.hardwareConcurrency')}</Label>
            <NumberSelector
              value={deviceSettings.hardwareConcurrency}
              options={[2, 4, 8, 16]}
              onChange={(v) =>
                onDeviceSettingsChange({ ...deviceSettings, hardwareConcurrency: v })
              }
              unit={t('deviceSettings.cores')}
            />
          </div>
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.deviceMemory')}</Label>
            <NumberSelector
              value={deviceSettings.deviceMemory}
              options={[
                4,
                8,
                // 16,
                // 32,
              ]}
              onChange={(v) => onDeviceSettingsChange({ ...deviceSettings, deviceMemory: v })}
              unit="G"
            />
          </div>
        </div>

        {/* 设备名称 */}
        <div className="space-y-1">
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.deviceName')}</Label>
            <SegmentedControl
              value={deviceSettings.deviceNameRandom ? 'random' : 'real'}
              options={deviceNameModeOptions}
              onChange={(v) =>
                onDeviceSettingsChange({ ...deviceSettings, deviceNameRandom: v === 'random' })
              }
            />
          </div>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{deviceSettings.deviceName}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() =>
                onDeviceSettingsChange({
                  ...deviceSettings,
                  deviceName: generateRandomDeviceName(),
                })
              }
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        {/* MAC地址 */}
        <div className="space-y-1">
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.macAddress')}</Label>
            <SegmentedControl
              value={deviceSettings.macAddressMode}
              options={macAddressModeOptions}
              onChange={(v) => onDeviceSettingsChange({ ...deviceSettings, macAddressMode: v })}
            />
          </div>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{deviceSettings.macAddress}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() =>
                onDeviceSettingsChange({
                  ...deviceSettings,
                  macAddress: generateRandomMacAddress(),
                })
              }
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
