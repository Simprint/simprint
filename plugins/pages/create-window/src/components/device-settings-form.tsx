import { useTranslation } from 'react-i18next';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { TextareaInput } from '@/components/textarea-input';
import { RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { DeviceSettings } from '../types';

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

interface DeviceSettingsFormProps {
  value: DeviceSettings;
  onChange: (value: DeviceSettings) => void;
}

export function DeviceSettingsForm({ value, onChange }: DeviceSettingsFormProps) {
  const { t } = useTranslation('create-window');

  const deviceNameModeOptions = [
    { value: 'random' as const, label: t('advancedFingerprint.fingerprintMode.random') },
    { value: 'real' as const, label: t('advancedFingerprint.fingerprintMode.real') },
  ];

  const macAddressModeOptions = [
    { value: 'real' as const, label: t('deviceSettings.macAddressMode.real') },
    { value: 'custom' as const, label: t('deviceSettings.macAddressMode.custom') },
  ];

  return (
    <div className="space-y-4 p-4 border border-border rounded-lg">
      <h3 className="text-sm font-semibold">{t('deviceSettings.title')}</h3>

      {/* 设备名称 & MAC地址 - 两列布局 */}
      <div className="grid grid-cols-2 gap-x-12 gap-y-4">
        <div className="space-y-1">
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.deviceName')}</Label>
            <SegmentedControl
              value={value.deviceNameRandom ? 'random' : 'real'}
              options={deviceNameModeOptions}
              onChange={(v) => onChange({ ...value, deviceNameRandom: v === 'random' })}
            />
          </div>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{value.deviceName}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() => onChange({ ...value, deviceName: generateRandomDeviceName() })}
              title={t('deviceSettings.refresh')}
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
        <div className="space-y-1">
          <div className="flex items-center justify-between gap-4">
            <Label className="text-xs shrink-0">{t('deviceSettings.macAddress')}</Label>
            <SegmentedControl
              value={value.macAddressMode}
              options={macAddressModeOptions}
              onChange={(v) => onChange({ ...value, macAddressMode: v })}
            />
          </div>
          <div className="flex items-center gap-1">
            <div className="flex-1 h-8 px-2 flex items-center text-xs font-mono bg-muted rounded overflow-hidden">
              <span className="truncate">{value.macAddress}</span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={() => onChange({ ...value, macAddress: generateRandomMacAddress() })}
              title={t('deviceSettings.refresh')}
            >
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>

      {/* 硬件并数 & 设备内存 - 两列布局 */}
      <div className="grid grid-cols-2 gap-x-12 gap-y-4">
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.hardwareConcurrency')}</Label>
          <NumberSelector
            value={value.hardwareConcurrency}
            options={[2, 4, 8, 16]}
            onChange={(v) => onChange({ ...value, hardwareConcurrency: v })}
            unit={t('deviceSettings.cores')}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.deviceMemory')}</Label>
          <NumberSelector
            value={value.deviceMemory}
            options={[
              4,
              8,
              // 16,
              // 32,
            ]}
            onChange={(v) => onChange({ ...value, deviceMemory: v })}
            unit="G"
          />
        </div>
      </div>

      {/* 开关选项 - Switch 风格 */}
      <div className="space-y-3 pt-2 border-t border-border">
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.sslFingerprint')}</Label>
          <Switch
            checked={value.sslFingerprint}
            onCheckedChange={(checked) => onChange({ ...value, sslFingerprint: checked })}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.portScanProtection')}</Label>
          <Switch
            checked={value.portScanProtection}
            onCheckedChange={(checked) => onChange({ ...value, portScanProtection: checked })}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.hardwareAcceleration')}</Label>
          <Switch
            checked={value.hardwareAcceleration}
            onCheckedChange={(checked) => onChange({ ...value, hardwareAcceleration: checked })}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <Label className="text-xs shrink-0">{t('deviceSettings.disableSandbox')}</Label>
          <Switch
            checked={value.disableSandbox}
            onCheckedChange={(checked) => onChange({ ...value, disableSandbox: checked })}
          />
        </div>
      </div>

      {/* 启动参数 */}
      <div className="pt-2 border-t border-border">
        <div className="space-y-1">
          <Label htmlFor="startup-parameters" className="text-xs">
            {t('deviceSettings.startupParameters')}
          </Label>
          <TextareaInput
            id="startup-parameters"
            value={value.startupParameters}
            onChange={(e) => onChange({ ...value, startupParameters: e.target.value })}
            className="text-xs bg-muted border-0"
            placeholder={t('deviceSettings.startupParametersPlaceholder')}
          />
        </div>
      </div>
    </div>
  );
}
