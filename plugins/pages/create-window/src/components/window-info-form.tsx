import { useState, useEffect, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { TextareaInput } from '@/components/textarea-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { RefreshCw, X, Globe, Facebook, Youtube, Instagram, Linkedin, Twitter } from 'lucide-react';
import { FiChrome } from 'react-icons/fi';
import { FaFirefoxBrowser, FaWindows, FaApple, FaLinux, FaGoogle, FaAmazon } from 'react-icons/fa';
import { SiGooglechrome } from 'react-icons/si';
import { toast } from 'sonner';
import { cn } from '@/lib/utils';
import type { CookieGroup, WindowInfo } from '../types';
import { CreateWindowSelectAccountDialog } from './create-window-select-account-dialog';
import { CreateWindowEditCookiesDialog } from './create-window-edit-cookies-dialog';
import { CreateWindowEditUrlsDialog } from './create-window-edit-urls-dialog';
import { generateUserAgentByKernel, parseUserAgent } from '../utils/user-agent-generator';
import { listBrowserKernels, type BrowserKernelVersion } from '../api';
// @ts-ignore - Cross-plugin import
import { listAccounts, type AccountItem } from '../../../environment-manager/src/api';

// 根据平台 URL 获取图标组件
function getPlatformIcon(platformUrl: string): React.FC<{ className?: string }> {
  const lower = platformUrl.toLowerCase();
  if (lower.includes('facebook.com') || lower.includes('fb.com')) {
    return () => <Facebook className="h-4 w-4 text-blue-600" />;
  }
  if (lower.includes('youtube.com')) {
    return () => <Youtube className="h-4 w-4 text-red-600" />;
  }
  if (lower.includes('instagram.com')) {
    return () => <Instagram className="h-4 w-4 text-pink-600" />;
  }
  if (lower.includes('linkedin.com')) {
    return () => <Linkedin className="h-4 w-4 text-blue-700" />;
  }
  if (lower.includes('twitter.com') || lower.includes('x.com')) {
    return () => <Twitter className="h-4 w-4 text-sky-500" />;
  }
  if (lower.includes('google.com') || lower.includes('gmail.com')) {
    return () => <FaGoogle className="h-4 w-4 text-blue-600" />;
  }
  if (lower.includes('amazon.com')) {
    return () => <FaAmazon className="h-4 w-4 text-orange-500" />;
  }
  return () => <Globe className="h-4 w-4 text-muted-foreground" />;
}

interface WindowInfoFormProps {
  value: WindowInfo;
  onChange: (value: WindowInfo) => void;
}

/**
 * 从内核版本列表中获取当前选中内核的完整版本号
 * 优先从 name 字段提取版本号（如 "simprint-browser-144.0.7559.118.zip"），
 * 如果提取失败则使用 version 字段
 */
function getCurrentKernelVersion(
  kernelVersions: BrowserKernelVersion[],
  currentKernelIdOrName: string
): string | undefined {
  const kernel = kernelVersions.find(
    (v) => v.kernel_id === currentKernelIdOrName || v.resource_name === currentKernelIdOrName
  );
  if (!kernel) {
    return undefined;
  }

  // 尝试从 name 字段提取版本号（格式：simprint-browser-144.0.7559.118.zip）
  if (kernel.name) {
    const match = kernel.name.match(/(\d+\.\d+\.\d+\.\d+)/);
    if (match) {
      return match[1];
    }
  }

  // 如果提取失败，使用 version 字段
  return kernel.version;
}

const systemOptions = [
  { value: 'Windows', label: 'Windows', icon: FaWindows },
  { value: 'macOS', label: 'macOS', icon: FaApple },
  { value: 'Linux', label: 'Linux', icon: FaLinux },
];

/** 系统到平台映射 */
const systemToPlatform: Record<string, string> = {
  Windows: 'windows',
  macOS: 'darwin',
  Linux: 'linux',
};

const KERNEL_TYPE_CHROME = 'chrome';
const KERNEL_TYPE_FIREFOX = 'firefox';
const SIMPRINT_KERNEL_CHROMIUM = 'SIMPRINT_KERNEL_CHROMIUM';

export function WindowInfoForm({ value, onChange }: WindowInfoFormProps) {
  const { t } = useTranslation('create-window');
  const [urlDialogOpen, setUrlDialogOpen] = useState(false);
  const [cookieDialogOpen, setCookieDialogOpen] = useState(false);
  const [accountDialogOpen, setAccountDialogOpen] = useState(false);
  const [allAccounts, setAllAccounts] = useState<AccountItem[]>([]);

  // 浏览器内核：chrome | firefox，默认 chrome
  const [kernelType, setKernelType] = useState<'chrome' | 'firefox'>(KERNEL_TYPE_CHROME);
  const [kernelVersions, setKernelVersions] = useState<BrowserKernelVersion[]>([]);
  const [kernelLoading, setKernelLoading] = useState(false);

  const hasSetDefaultKernel = useRef<Record<string, boolean>>({});

  // 加载内核版本列表（仅 Chrome 支持）
  // 注意：所有系统都使用 Windows 平台的内核版本，系统选择只影响 UA 格式
  // 只在 kernelType 变化时加载一次
  useEffect(() => {
    if (kernelType !== KERNEL_TYPE_CHROME) {
      setKernelVersions([]);
      return;
    }
    // 始终使用 Windows 平台的内核版本列表
    const platform = 'windows';
    setKernelLoading(true);
    listBrowserKernels(platform, SIMPRINT_KERNEL_CHROMIUM)
      .then((data) => {
        const versions = data[SIMPRINT_KERNEL_CHROMIUM] || [];
        setKernelVersions(versions);
        // 首次加载且 kernel 不在列表中时，设为第一个版本
        if (versions.length > 0) {
          const currentInList = versions.some(
            (v) => v.kernel_id === value.kernelId || (!value.kernelId && v.resource_name === value.kernel)
          );
          const needDefault =
            !hasSetDefaultKernel.current[platform] &&
            (!currentInList || value.kernel === 'Chrome');
          if (needDefault) {
            hasSetDefaultKernel.current[platform] = true;
            const firstResourceName = versions[0].resource_name;
            const kernelVersion = versions[0].version;
            const newUA = generateUserAgentByKernel(value.system, 'Chrome', kernelVersion);
            onChange({
              ...value,
              kernel: firstResourceName,
              kernelId: versions[0].kernel_id,
              userAgent: newUA,
            });
          }
        }
      })
      .catch(() => setKernelVersions([]))
      .finally(() => setKernelLoading(false));
  }, [kernelType]); // 只依赖 kernelType，不依赖 value.system

  const handleChromeClick = () => {
    setKernelType(KERNEL_TYPE_CHROME);
  };

  const handleFirefoxClick = () => {
    toast.info(t('windowInfo.firefoxNotSupported'));
  };

  const handleKernelVersionChange = (kernelId: string) => {
    const selected = kernelVersions.find((kernel) => kernel.kernel_id === kernelId);
    if (!selected) return;
    const kernelVersion = getCurrentKernelVersion(kernelVersions, kernelId);
    const newUA = generateUserAgentByKernel(value.system, 'Chrome', kernelVersion);
    onChange({
      ...value,
      kernel: selected.resource_name,
      kernelId: selected.kernel_id,
      userAgent: newUA,
    });
  };

  // 加载账号列表（用于显示）
  useEffect(() => {
    listAccounts()
      .then(setAllAccounts)
      .catch(() => {
        // 忽略错误
      });
  }, []);

  // 获取选中的账号信息
  const selectedAccounts = useMemo(() => {
    if (!value.accountUuids || value.accountUuids.length === 0) return [];
    return allAccounts.filter((acc) => value.accountUuids.includes(acc.uuid));
  }, [value.accountUuids, allAccounts]);

  const handleAccountConfirm = (accountUuids: string[]) => {
    onChange({
      ...value,
      accountUuids: accountUuids,
    });
  };

  const handleCookiesConfirm = (cookies: CookieGroup[]) => {
    onChange({
      ...value,
      cookies: cookies,
    });
  };

  const handleUrlsConfirm = (urls: string[]) => {
    onChange({
      ...value,
      urls: urls,
    });
  };

  return (
    <div className="relative space-y-4 p-4 border border-border rounded-lg overflow-hidden min-w-0">
      {/* 右上角 Chrome 背景装饰图标 */}
      <div className="absolute -top-28 -right-28 pointer-events-none">
        <SiGooglechrome className="w-96 h-96 text-muted-foreground/10" />
      </div>

      <h3 className="text-sm font-semibold">{t('sections.basicInfo')}</h3>

      {/* 窗口名称 & 备注 - 两列 */}
      <div className="grid grid-cols-2 gap-x-8 gap-y-4">
        <div className="space-y-2">
          <Label htmlFor="window-name" className="text-xs">
            {t('windowInfo.name')}
          </Label>
          <TextareaInput
            id="window-name"
            value={value.name}
            onChange={(e) => onChange({ ...value, name: e.target.value })}
            placeholder={t('windowInfo.namePlaceholder')}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="description" className="text-xs">
            {t('windowInfo.description')}
          </Label>
          <TextareaInput
            id="description"
            value={value.description}
            onChange={(e) => onChange({ ...value, description: e.target.value })}
            className="text-xs"
            placeholder={t('windowInfo.descriptionPlaceholder')}
          />
        </div>
      </div>

      {/* 系统选择 & 浏览器选择 - 同一行 */}
      <div className="grid grid-cols-2 gap-x-8 gap-y-4">
        <div className="space-y-2">
          <Label className="text-xs">{t('windowInfo.system')}</Label>
          <div className="flex gap-1">
            {systemOptions.map((option) => {
              const IconComponent = option.icon;
              return (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => {
                    const kernelVersion = getCurrentKernelVersion(
                      kernelVersions,
                      value.kernelId || value.kernel
                    );
                    const newUA = generateUserAgentByKernel(option.value, 'Chrome', kernelVersion);
                    onChange({ ...value, system: option.value, userAgent: newUA });
                  }}
                  className={cn(
                    'flex items-center gap-1.5 px-3 py-1.5 text-xs rounded transition-all',
                    value.system === option.value
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  )}
                >
                  <IconComponent />
                  {option.label}
                </button>
              );
            })}
          </div>
        </div>
        <div className="space-y-2">
          <Label className="text-xs">{t('windowInfo.browser')}</Label>
          <div className="flex items-center gap-1">
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  onClick={handleChromeClick}
                  className={cn(
                    'flex items-center justify-center w-9 h-9 rounded transition-all',
                    kernelType === KERNEL_TYPE_CHROME
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  )}
                >
                  <FiChrome className="w-4 h-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent>Chrome / Chromium</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  onClick={handleFirefoxClick}
                  className={cn(
                    'flex items-center justify-center w-9 h-9 rounded transition-all',
                    kernelType === KERNEL_TYPE_FIREFOX
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80 opacity-60'
                  )}
                >
                  <FaFirefoxBrowser className="w-4 h-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent>{t('windowInfo.firefoxNotSupported')}</TooltipContent>
            </Tooltip>
            <Select
              value={
                kernelVersions.some((v) => v.kernel_id === value.kernelId)
                  ? value.kernelId
                  : kernelVersions.find((v) => v.resource_name === value.kernel)?.kernel_id ||
                    kernelVersions[0]?.kernel_id ||
                    ''
              }
              onValueChange={handleKernelVersionChange}
              disabled={kernelLoading || kernelVersions.length === 0}
            >
              <SelectTrigger className="h-9 min-w-[160px] flex-1">
                <SelectValue
                  placeholder={
                    kernelLoading ? '加载中...' : kernelVersions.length === 0 ? '暂无版本' : '选择版本'
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {kernelVersions.map((v) => (
                  <SelectItem key={v.kernel_id} value={v.kernel_id}>
                    {v.resource_name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* User-Agent（系统选择下方） */}
      <div className="space-y-2">
        <Label className="text-xs">{t('windowInfo.userAgent')}</Label>
        <div className="flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="w-[32rem] shrink-0 h-9 px-3 flex items-center text-xs font-mono bg-muted rounded overflow-hidden cursor-help">
                <span className="block truncate min-w-0">{value.userAgent}</span>
              </div>
            </TooltipTrigger>
            <TooltipContent side="bottom" align="start" className="max-w-md">
              <div className="space-y-2">
                <p className="font-mono text-xs break-all">{value.userAgent}</p>
                {(() => {
                  const info = parseUserAgent(value.userAgent);
                  return (
                    <div className="text-xs border-t border-white/10 pt-2 space-x-2">
                      <span className="text-zinc-400">浏览器:</span>
                      <span className="font-medium text-zinc-100">
                        {info.browser}{' '}
                        {info.browserVersion ? info.browserVersion.split('.')[0] : ''}
                      </span>
                      <span className="text-zinc-400">操作系统:</span>
                      <span className="font-medium text-zinc-100">
                        {info.os} {info.osVersion || ''}
                      </span>
                    </div>
                  );
                })()}
              </div>
            </TooltipContent>
          </Tooltip>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-9 w-9 shrink-0"
            onClick={() => {
              const uaKernel = kernelType === KERNEL_TYPE_CHROME ? 'Chrome' : 'Firefox';
              const kernelVersion = getCurrentKernelVersion(
                kernelVersions,
                value.kernelId || value.kernel
              );
              const newUA = generateUserAgentByKernel(value.system, uaKernel, kernelVersion);
              onChange({ ...value, userAgent: newUA });
            }}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* 账号与数据分隔线 */}
      <div className="pt-2 border-t border-border space-y-4">
        {/* 平台账号 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-xs">{t('windowInfo.platformAccount')}</Label>
            <Button
              variant="outline"
              size="sm"
              className="h-7 text-xs"
              onClick={() => setAccountDialogOpen(true)}
            >
              {t('windowInfo.platformAccountActions.set') || '设置'}
            </Button>
          </div>
          {selectedAccounts.length > 0 ? (
            <div className="space-y-1">
              {selectedAccounts.map((account) => {
                const PlatformIcon = getPlatformIcon(account.platform_url || '');
                return (
                  <div key={account.uuid} className="flex items-center gap-1">
                    <div className="flex-1 h-8 px-2 flex items-center gap-2 text-xs bg-muted rounded overflow-hidden">
                      <PlatformIcon />
                      <span className="truncate">
                        {account.platform_name || account.platform_url} - {account.account}
                      </span>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 shrink-0"
                      onClick={() => {
                        onChange({
                          ...value,
                          accountUuids: value.accountUuids.filter((uuid) => uuid !== account.uuid),
                        });
                      }}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="h-8 px-3 flex items-center text-xs text-muted-foreground bg-muted rounded">
              {t('windowInfo.noPlatformAccount')}
            </div>
          )}
        </div>

        {/* URL & Cookie - 两列 */}
        <div className="grid grid-cols-2 gap-x-8 gap-y-4">
          {/* URL */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('windowInfo.urls')}</Label>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs"
                onClick={() => setUrlDialogOpen(true)}
              >
                {t('windowInfo.urlActions.set') || '设置'}
              </Button>
            </div>
            {value.urls.length > 0 ? (
              <div className="space-y-1">
                {value.urls.map((url, index) => (
                  <div key={index} className="flex items-center gap-1">
                    <div className="flex-1 h-8 px-2 flex items-center text-xs bg-muted rounded overflow-hidden">
                      <span className="truncate">{url}</span>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 shrink-0"
                      onClick={() => {
                        onChange({ ...value, urls: value.urls.filter((_, i) => i !== index) });
                      }}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="h-8 px-3 flex items-center text-xs text-muted-foreground bg-muted rounded">
                {t('windowInfo.noUrls')}
              </div>
            )}
          </div>

          {/* Cookie */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('windowInfo.cookies')}</Label>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs"
                onClick={() => setCookieDialogOpen(true)}
              >
                {t('windowInfo.cookieActions.set') || '设置'}
              </Button>
            </div>
            {value.cookies.length > 0 ? (
              <div className="space-y-1">
                {value.cookies.map((cookie, index) => (
                  <div key={index} className="flex items-center gap-1">
                    <div className="flex-1 h-8 px-2 flex items-center text-xs bg-muted rounded overflow-hidden">
                      <div className="truncate font-medium">{cookie.site}</div>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 shrink-0"
                      onClick={() => {
                        onChange({
                          ...value,
                          cookies: value.cookies.filter((_, i) => i !== index),
                        });
                      }}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="h-8 px-3 flex items-center text-xs text-muted-foreground bg-muted rounded">
                {t('windowInfo.noCookies')}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* URL 设置对话框 */}
      <CreateWindowEditUrlsDialog
        open={urlDialogOpen}
        urls={value.urls || []}
        onOpenChange={setUrlDialogOpen}
        onConfirm={handleUrlsConfirm}
      />

      {/* Cookie 设置对话框 */}
      <CreateWindowEditCookiesDialog
        open={cookieDialogOpen}
        cookies={value.cookies || []}
        onOpenChange={setCookieDialogOpen}
        onConfirm={handleCookiesConfirm}
      />

      {/* 账号设置对话框 */}
      <CreateWindowSelectAccountDialog
        open={accountDialogOpen}
        selectedAccountUuids={value.accountUuids || []}
        onOpenChange={(open) => {
          setAccountDialogOpen(open);
          // 对话框关闭时重新加载账号列表（可能创建了新账号）
          if (!open) {
            listAccounts()
              .then(setAllAccounts)
              .catch(() => {
                // 忽略错误
              });
          }
        }}
        onConfirm={handleAccountConfirm}
      />
    </div>
  );
}
