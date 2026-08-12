import { useEffect, useRef, useState } from 'react';
import {
  Network,
  Workflow,
  TerminalSquare,
  ShieldHalf,
  FolderTree,
  ChevronLeft,
  ChevronRight,
  Puzzle,
  Users,
  UserCircle,
} from 'lucide-react';
import { TfiWorld } from "react-icons/tfi";
import { BsWindowSidebar } from "react-icons/bs";
import { VscGitPullRequestCreate } from "react-icons/vsc";
import type { IconType } from 'react-icons'
import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';
import { Link, useLocation, useNavigate } from 'react-router';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useTranslation } from 'react-i18next';
import { getGeneralSettings, useMihomoRuntimeStore } from '../../../../services/store/src';
import { ClashIcon } from '../../../../pages/proxy-center/src/mihomo/clash-icon';
import { MihomoConnectDialog } from '../../../../pages/proxy-center/src/mihomo/mihomo-connect-dialog';

interface NavItemData {
  label: string;
  shortLabel?: string;
  href: string;
  icon: LucideIcon | IconType;
}

interface NavGroup {
  title?: string;
  items: NavItemData[];
}

function localizeNavGroups(t: (key: string) => string): NavGroup[] {
  return [
    {
      title: t('nav.group.basic'),
      items: [
        {
          label: t('nav.item.proxy'),
          shortLabel: t('nav.short.proxy'),
          href: '/proxy',
          icon: Network,
        },
        {
          label: t('nav.item.accounts'),
          shortLabel: t('nav.short.accounts'),
          href: '/accounts',
          icon: UserCircle,
        },
        {
          label: t('nav.item.extensions'),
          shortLabel: t('nav.short.extensions'),
          href: '/extensions',
          icon: Puzzle,
        },
      ],
    },
    {
      title: t('nav.group.automation'),
      items: [
        { label: t('nav.item.rpa'), shortLabel: t('nav.short.rpa'), href: '/rpa', icon: Workflow },
        {
          label: t('nav.item.api'),
          shortLabel: t('nav.short.api'),
          href: '/api',
          icon: TerminalSquare,
        },
      ],
    },
    {
      title: t('nav.group.collaboration'),
      items: [
        { label: t('nav.item.team'), shortLabel: t('nav.short.team'), href: '/team', icon: Users },
      ],
    },
  ];
}

interface NavItemProps {
  label: string;
  href: string;
  icon: LucideIcon | IconType;
  isActive: boolean;
  collapsed: boolean;
  actionIcon?: LucideIcon | IconType;
  actionNode?: ReactNode;
  actionLabel?: string;
  actionButtonClassName?: string;
  actionButtonAnimated?: boolean;
  onActionClick?: () => void;
}

const COLLAPSED_PROXY_ACTION_DELAY_MS = 1000;

function isRouteActive(currentPath: string, href: string): boolean {
  return currentPath === href || currentPath.startsWith(`${href}/`);
}

/**
 * 导航项组件
 */
const NavItem: React.FC<NavItemProps> = ({
  label,
  href,
  icon: Icon,
  isActive,
  collapsed,
  actionIcon: ActionIcon,
  actionNode,
  actionLabel,
  actionButtonClassName,
  actionButtonAnimated,
  onActionClick,
}) => {
  const [collapsedActionVisible, setCollapsedActionVisible] = useState(false);
  const hoverTimerRef = useRef<number | null>(null);
  const hasAction = !!((ActionIcon || actionNode) && onActionClick);

  useEffect(() => {
    if (!collapsed || !hasAction) {
      setCollapsedActionVisible(false);
    }
  }, [collapsed, hasAction]);

  useEffect(() => () => {
    if (hoverTimerRef.current != null) {
      window.clearTimeout(hoverTimerRef.current);
    }
  }, []);

  const clearHoverTimer = () => {
    if (hoverTimerRef.current != null) {
      window.clearTimeout(hoverTimerRef.current);
      hoverTimerRef.current = null;
    }
  };

  const handleMouseEnter = () => {
    if (!collapsed || !hasAction) {
      return;
    }

    clearHoverTimer();
    hoverTimerRef.current = window.setTimeout(() => {
      setCollapsedActionVisible(true);
      hoverTimerRef.current = null;
    }, COLLAPSED_PROXY_ACTION_DELAY_MS);
  };

  const handleMouseLeave = () => {
    clearHoverTimer();
    setCollapsedActionVisible(false);
  };

  return (
    <div
      className={`group relative ${collapsed ? 'w-11' : 'w-full'}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {isActive && <div className='absolute -right-1 top-1/2 h-6 w-1 -translate-y-1/2 rounded-2xl bg-primary'>&nbsp;</div>}
      <div
        className={`relative rounded-lg h-11 text-sm transition-[padding,gap] duration-300 ease-in-out ${isActive
          ? 'bg-primary/15 text-primary font-semibold border-primary'
          : 'text-sidebar-foreground/80 hover:bg-accent/60 hover:text-foreground border-transparent'
          }`}
      >
        <Link
          to={href}
          className={`flex h-11 items-center ${collapsed
            ? 'justify-center px-3 w-11'
            : `${(ActionIcon || actionNode) && onActionClick ? 'pr-11' : 'pr-3'} pl-3 gap-3`
            }`}
        >
          <Tooltip>
            <TooltipTrigger asChild>
              <Icon
                className={`h-4 w-4 transform-gpu ${isActive ? 'text-primary' : 'text-sidebar-foreground opacity-80'
                  } transition-all duration-200 ${collapsed && hasAction
                    ? collapsedActionVisible
                      ? '-translate-x-2 opacity-0'
                      : 'translate-x-0 opacity-100'
                    : ''
                  }`}
              />
            </TooltipTrigger>
            <TooltipContent side="right">{label}</TooltipContent>
          </Tooltip>

          <span
            className={`text-xs h-4 overflow-hidden font-semibold transition-all duration-300 ${isActive ? 'text-primary' : 'text-sidebar-foreground/90'
              } ${collapsed ? 'max-w-0 hidden' : 'max-w-50 block'}`}
          >
            {label}
          </span>
        </Link>

        {!collapsed && hasAction && (
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="absolute right-1 top-1/2 -translate-y-1/2">
                <button
                  type="button"
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    onActionClick();
                  }}
                  className={`flex h-8 w-8 items-center justify-center rounded-md border bg-background/60 transition-colors ${actionButtonAnimated ? 'animate-[sidebar-proxy-action-drift_10s_ease-in-out_infinite]' : ''} ${actionButtonClassName || 'border-border/60 text-sidebar-foreground/80 hover:bg-accent hover:text-foreground'}`}
                  aria-label={actionLabel || label}
                >
                  {actionNode || (ActionIcon ? <ActionIcon className="h-3.5 w-3.5" /> : null)}
                </button>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">{actionLabel || label}</TooltipContent>
          </Tooltip>
        )}

        {collapsed && hasAction && (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  onActionClick();
                }}
                className={`absolute inset-0 z-10 flex h-11 w-11 items-center justify-center rounded-lg border transform-gpu transition-all duration-200 ${collapsedActionVisible
                  ? 'pointer-events-auto translate-x-0 opacity-100'
                  : 'pointer-events-none translate-x-2 opacity-0'
                  } ${actionButtonClassName || (isActive
                    ? 'border-primary/40 bg-primary/10 text-primary'
                    : 'border-border/60 bg-background/90 text-sidebar-foreground/80 hover:bg-accent hover:text-foreground')
                  }`}
                aria-label={actionLabel || label}
              >
                <span className="flex items-center justify-center transition-transform duration-200">
                  {actionNode || (ActionIcon ? <ActionIcon className="h-4 w-4" /> : null)}
                </span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{actionLabel || label}</TooltipContent>
          </Tooltip>
        )}
      </div>
    </div>
  );
};

interface NavGroupComponentProps {
  group: NavGroup;
  collapsed: boolean;
  currentPath: string;
  proxyActionLabel: string;
  proxyActionNode: ReactNode;
  proxyActionButtonClassName?: string;
  proxyActionButtonAnimated?: boolean;
  onProxyActionClick: () => void;
}

/**
 * 导航分组组件
 */
const NavGroupComponent: React.FC<NavGroupComponentProps> = ({
  group,
  collapsed,
  currentPath,
  proxyActionLabel,
  proxyActionNode,
  proxyActionButtonClassName,
  proxyActionButtonAnimated,
  onProxyActionClick,
}) => {
  return (
    <div className="space-y-1">
      {group.title && !collapsed && (
        <div className="px-3 py-1 text-[10px] font-semibold text-sidebar-foreground/50 uppercase tracking-wider">
          {group.title}
        </div>
      )}
      {group.items.map((item) => (
        <NavItem
          key={item.href}
          label={item.label}
          href={item.href}
          icon={item.icon}
          isActive={isRouteActive(currentPath, item.href)}
          collapsed={collapsed}
          actionNode={item.href === '/proxy' ? proxyActionNode : undefined}
          actionLabel={item.href === '/proxy' ? proxyActionLabel : undefined}
          actionButtonClassName={item.href === '/proxy' ? proxyActionButtonClassName : undefined}
          actionButtonAnimated={item.href === '/proxy' ? proxyActionButtonAnimated : undefined}
          onActionClick={item.href === '/proxy' ? onProxyActionClick : undefined}
        />
      ))}
    </div>
  );
};

interface NavListProps {
  groups: NavGroup[];
  collapsed: boolean;
  currentPath: string;
  proxyActionLabel: string;
  proxyActionNode: ReactNode;
  proxyActionButtonClassName?: string;
  proxyActionButtonAnimated?: boolean;
  onProxyActionClick: () => void;
}

/**
 * 导航列表组件
 */
const NavList: React.FC<NavListProps> = ({
  groups,
  collapsed,
  currentPath,
  proxyActionLabel,
  proxyActionNode,
  proxyActionButtonClassName,
  proxyActionButtonAnimated,
  onProxyActionClick,
}) => {
  return (
    <ScrollArea className="flex-1 m-1 min-h-0">
      <nav>
        {groups.map((group, index) => (
          <div key={group.title || `group-${index}`}>
            {index > 0 && collapsed && <div className="my-1 mx-3 border-t border-sidebar-border" />}
            <div className={collapsed ? '' : 'mb-3'}>
              <NavGroupComponent
                group={group}
                collapsed={collapsed}
                currentPath={currentPath}
                proxyActionLabel={proxyActionLabel}
                proxyActionNode={proxyActionNode}
                proxyActionButtonClassName={proxyActionButtonClassName}
                proxyActionButtonAnimated={proxyActionButtonAnimated}
                onProxyActionClick={onProxyActionClick}
              />
            </div>
          </div>
        ))}
      </nav>
    </ScrollArea>
  );
};

interface CollapseButtonProps {
  collapsed: boolean;
  onToggle: () => void;
}

/**
 * 折叠按钮组件
 */
const CollapseButton: React.FC<CollapseButtonProps> = ({ collapsed, onToggle }) => {
  const { t } = useTranslation('appLayout');
  const Icon = collapsed ? ChevronRight : ChevronLeft;

  return (
    <Tooltip>
      <TooltipTrigger
        asChild
        onClick={onToggle}
        className="absolute bottom-0 -right-6 z-20 flex items-center justify-center text-sidebar-foreground/70 hover:text-foreground transition-colors duration-200 animate-[pulse-gentle_2s_ease-in-out_infinite] group cursor-pointer"
      >
        <Icon className="-rotate-45 transition-transform duration-200 opacity-80 group-hover:opacity-100 animate-[slide-hint_2s_ease-in-out_infinite]" />
      </TooltipTrigger>
      <TooltipContent side="right">
        {collapsed ? t('nav.sidebar.expand') : t('nav.sidebar.collapse')}
      </TooltipContent>
    </Tooltip>
  );
};

interface CreateEnvironmentButtonProps {
  collapsed: boolean;
}

/**
 * 创建环境按钮组件
 */
const CreateEnvironmentButton: React.FC<CreateEnvironmentButtonProps> = ({ collapsed }) => {
  const { t } = useTranslation('appLayout');
  const navigate = useNavigate();
  const handleCreateEnvironment = () => {
    navigate('/create-window');
  };

  const buttonContent = (
    <button
      type="button"
      onClick={handleCreateEnvironment}
      className={`group relative w-full rounded-lg mb-2 text-sm transition-all duration-300 ease-in-out overflow-hidden cursor-pointer ${collapsed
        ? 'flex flex-col items-center py-3.5'
        : 'flex flex-row items-center gap-3 px-3 py-4'
        } bg-linear-to-r from-blue-500 via-blue-600 to-indigo-600 hover:from-blue-600 hover:via-blue-700 hover:to-indigo-700 text-white font-semibold border-blue-500 shadow-sm hover:shadow-md`}
    >
      {/* 背景装饰图标 - 仅在展开状态下显示 */}
      {!collapsed && (
        <>
          <TfiWorld className="absolute -right-2 -top-2 h-16 w-16 text-sidebar-primary-foreground/10 transition-all duration-200" />
          {/* <TfiWorld className="absolute right-3 top-3 h-14 w-14 text-sidebar-primary-foreground/10 transition-all duration-200" /> */}
        </>
      )}

      {/* 前景内容 */}
      <div className={`relative z-10 flex items-center ${collapsed ? '' : 'gap-3'}`}>
        <VscGitPullRequestCreate
          className={`h-4 w-4 text-sidebar-primary-foreground transition-all duration-200`}
        />

        <span
          className={`text-xs font-semibold text-sidebar-primary-foreground h-4 overflow-hidden transition-all duration-300 ${collapsed ? 'max-w-0 hidden' : 'max-w-[200px] block'
            }`}
        >
          {t('nav.actions.createEnv')}
        </span>
      </div>
    </button>
  );

  // 折叠状态下使用 Tooltip 组件显示提示
  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{buttonContent}</TooltipTrigger>
        <TooltipContent side="right">{t('nav.actions.createEnv')}</TooltipContent>
      </Tooltip>
    );
  }

  return buttonContent;
};

interface TopNavItemsProps {
  collapsed: boolean;
  currentPath: string;
}

/**
 * 顶部导航项组件
 * 包含环境列表和分组管理
 */
const TopNavItems: React.FC<TopNavItemsProps> = ({ collapsed, currentPath }) => {
  const { t } = useTranslation('appLayout');
  const topNavItems: NavItemData[] = [
    { label: t('nav.item.envList'), href: '/', icon: BsWindowSidebar },
    { label: t('nav.item.groups'), href: '/groups', icon: FolderTree },
  ];

  return (
    <>
      {topNavItems.map((item) => (
        <NavItem
          key={item.href}
          label={item.label}
          href={item.href}
          icon={item.icon}
          isActive={isRouteActive(currentPath, item.href)}
          collapsed={collapsed}
        />
      ))}
    </>
  );
};

interface BottomNavItemsProps {
  collapsed: boolean;
  currentPath: string;
}

/**
 * 底部导航项组件
 * 底部保留本地审计入口。
 */
const BottomNavItems: React.FC<BottomNavItemsProps> = ({ collapsed, currentPath }) => {
  const { t } = useTranslation('appLayout');
  const bottomNavItems: NavItemData[] = [
    { label: t('nav.item.audit'), href: '/audit', icon: ShieldHalf },
  ];

  return (
    <div className="m-1 space-y-1">
      {collapsed && <div className="mx-3 border-t border-sidebar-border mb-1" />}
      {bottomNavItems.map((item) => (
        <NavItem
          key={item.href}
          label={item.label}
          href={item.href}
          icon={item.icon}
          isActive={isRouteActive(currentPath, item.href)}
          collapsed={collapsed}
        />
      ))}
    </div>
  );
};

// 动画时长常量
const SIDEBAR_ANIMATION_DURATION = 300;

interface SidebarPlaceholderProps {
  collapsed: boolean;
}

/**
 * 侧边栏占位符组件
 * 在动画期间显示，保持固定高度避免滚动条
 */
const SidebarPlaceholder: React.FC<SidebarPlaceholderProps> = ({ collapsed }) => {
  return (
    <div className="flex flex-col h-full animate-[fadeIn_300ms_ease-in-out]">
      {/* 顶部按钮区域占位符 */}
      <div className="mt-1 mb-2 m-1 space-y-1 flex flex-col overflow-hidden shrink-0">
        <div className="h-12 bg-sidebar rounded-lg" />
        <div className="h-10 bg-sidebar rounded-lg" />
        <div className="h-10 bg-sidebar rounded-lg" />
        {collapsed && <div className="mx-3 border-b border-sidebar-border pt-2" />}
      </div>

      {/* 导航列表区域占位符 */}
      <div className="flex-1 m-1 min-h-0 overflow-hidden">
        <div className="h-full bg-sidebar/30 rounded-lg" />
      </div>

      {/* 免费额度使用情况区域占位符 */}
      <div className="shrink-0">
        {collapsed ? (
          <div className="m-1 space-y-1">
            <div className="h-12 bg-sidebar rounded-lg" />
            <div className="h-12 bg-sidebar rounded-lg" />
            <div className="h-12 bg-sidebar rounded-lg" />
          </div>
        ) : (
          <div className="mx-1 mb-2">
            <div className="h-32 bg-sidebar border border-sidebar-border/80 rounded-lg" />
          </div>
        )}
      </div>
    </div>
  );
};

/**
 * 应用侧边栏组件
 */
export const AppSidebar: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useTranslation('appLayout');
  const [collapsed, setCollapsed] = useState(true);
  const [isAnimating, setIsAnimating] = useState(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const [mihomoDialogOpen, setMihomoDialogOpen] = useState(false);
  const mihomoRunning = useMihomoRuntimeStore((state) => state.running);
  const mihomoAttached = useMihomoRuntimeStore((state) => state.attached);
  const mihomoCheckedAt = useMihomoRuntimeStore((state) => state.checkedAt);

  useEffect(() => {
    let cancelled = false;

    void getGeneralSettings()
      .then((settings) => {
        if (!cancelled) {
          setCollapsed(settings.defaultCollapseSidebar);
          setSettingsLoaded(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSettingsLoaded(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const handleOpenMihomo = async () => {
    if (!mihomoRunning) {
      navigate('/proxy?mode=local');
      return;
    }

    await useMihomoRuntimeStore.getState().refresh();
    const { running, attached } = useMihomoRuntimeStore.getState();

    if (!running) {
      navigate('/proxy?mode=local');
      return;
    }

    if (attached) {
      navigate('/proxy/mihomo');
      return;
    }

    setMihomoDialogOpen(true);
  };

  const handleToggle = () => {
    setIsAnimating(true);
    setCollapsed((prev) => !prev);
    // 等待动画完成后清除动画状态
    setTimeout(() => {
      setIsAnimating(false);
    }, SIDEBAR_ANIMATION_DURATION);
  };

  if (!settingsLoaded) {
    return <SidebarPlaceholder collapsed={collapsed} />;
  }

  const mihomoStatusTone = mihomoRunning
    ? mihomoAttached
      ? 'connected'
      : 'warning'
    : mihomoCheckedAt == null
      ? 'unknown'
      : 'offline';

  const mihomoActionNode = (
    <span className="relative flex items-center justify-center">
      <ClashIcon className="h-4 w-4 text-current" />
      {mihomoStatusTone === 'connected' ? (
        <span className="absolute -right-1 -top-1 h-2.5 w-2.5 rounded-full border border-background bg-emerald-500" />
      ) : mihomoStatusTone === 'warning' ? (
        <span className="absolute -right-1 -top-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border border-background bg-amber-500 text-[9px] font-bold leading-none text-white">
          !
        </span>
      ) : mihomoStatusTone === 'unknown' ? (
        <span className="absolute -right-1 -top-1 h-2.5 w-2.5 rounded-full border border-background bg-muted-foreground/60" />
      ) : (
        <span className="absolute -right-1 -top-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border border-background bg-rose-500 text-[9px] font-bold leading-none text-white">
          !
        </span>
      )}
    </span>
  );

  return (
    <aside
      className={`flex flex-col h-full shrink-0 transition-[width] duration-300 ease-out relative before:absolute before:inset-0 before:bg-background/10 before:backdrop-blur-2xl before:-z-10 px-1 ${collapsed ? 'w-16' : 'w-58'
        }`}
    >
      <div className={`flex flex-col h-full ${isAnimating ? 'opacity-0' : 'animate-[slideInRight_300ms_ease-in-out]'}`}>
        {/* 顶部导航区域 */}
        <div className="mt-1 mb-2 m-1 space-y-1 flex flex-col shrink-0">
          <CreateEnvironmentButton collapsed={collapsed} />
          <TopNavItems collapsed={collapsed} currentPath={location.pathname} />
          {collapsed && <div className="mx-3 border-b border-sidebar-border pt-2" />}
        </div>

        {/* 导航列表区域 */}
      <NavList
        groups={localizeNavGroups(t)}
        collapsed={collapsed}
        currentPath={location.pathname}
        proxyActionLabel={t('nav.item.mihomo', { defaultValue: 'Mihomo' })}
        proxyActionNode={mihomoActionNode}
        proxyActionButtonClassName={
          mihomoStatusTone === 'connected'
            ? 'border-emerald-500/30 bg-background/90 text-emerald-600 hover:bg-muted hover:text-foreground'
            : mihomoStatusTone === 'warning'
              ? 'border-amber-500/30 bg-background/90 text-amber-600 hover:bg-muted hover:text-foreground'
              : 'border-rose-500/30 bg-background/90 text-rose-600 hover:bg-muted hover:text-rose-600'
        }
        proxyActionButtonAnimated={!collapsed}
        onProxyActionClick={handleOpenMihomo}
      />

        {/* 底部导航项（指纹审计、系统设置） */}
        <div className="shrink-0">
          <BottomNavItems collapsed={collapsed} currentPath={location.pathname} />
        </div>

        {/* 免费额度使用情况区域 */}
        <div className="shrink-0">
        </div>

        {/* 折叠按钮区域 */}
        <div className="shrink-0">
          <CollapseButton collapsed={collapsed} onToggle={handleToggle} />
        </div>
      </div>
      <MihomoConnectDialog
        open={mihomoDialogOpen}
        onOpenChange={setMihomoDialogOpen}
        onConnected={() => {
          void useMihomoRuntimeStore.getState().refresh();
          navigate('/proxy/mihomo');
        }}
      />
    </aside>
  );
};
