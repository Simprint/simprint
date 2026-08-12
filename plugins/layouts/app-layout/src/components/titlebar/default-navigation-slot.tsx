import {
  ChevronRight,
  Home,
  FolderTree,
  Network,
  UserCircle,
  Workflow,
  TerminalSquare,
  Puzzle,
  Users,
  ShieldHalf,
  Settings,
  SquarePlus,
} from 'lucide-react';
import { useLocation } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useSearchParams } from 'react-router';
import type { LucideIcon } from 'lucide-react';
import { BsWindowSidebar } from "react-icons/bs";
import type { IconType } from "react-icons";

const navItems: Array<{ key: string; href: string; icon: LucideIcon | IconType }> = [
  { key: 'envList', href: '/', icon: BsWindowSidebar },
  { key: 'groups', href: '/groups', icon: FolderTree },
  { key: 'proxy', href: '/proxy', icon: Network },
  { key: 'accounts', href: '/accounts', icon: UserCircle },
  { key: 'rpa', href: '/rpa', icon: Workflow },
  { key: 'api', href: '/api', icon: TerminalSquare },
  { key: 'extensions', href: '/extensions', icon: Puzzle },
  { key: 'team', href: '/team', icon: Users },
  { key: 'audit', href: '/audit', icon: ShieldHalf },
  { key: 'settings', href: '/settings', icon: Settings },
  { key: 'createWindow', href: '/create-window', icon: SquarePlus },
];

/**
 * 默认导航区域组件
 */
export function DefaultNavigationSlot() {
  const { t } = useTranslation('appLayout');
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const currentNav = navItems.find((item) => item.href === location.pathname);

  // 检查是否是编辑模板模式
  const isTemplateEditMode = location.pathname === '/create-window' && searchParams.get('template');
  // 检查是否是编辑环境模式
  const isEditEnvironmentMode = location.pathname === '/create-window' && searchParams.get('edit');

  return (
    <div className="flex items-center gap-2.5 px-5 text-xs text-muted-foreground">
      <Home className="w-3 h-3 opacity-70" />
      {currentNav && (
        <>
          <ChevronRight className="w-3 h-3 opacity-50" />
          <currentNav.icon className="w-3 h-3 opacity-70" />
          <span className="text-muted-foreground">
            {isTemplateEditMode
              ? t('navigation.editTemplate')
              : isEditEnvironmentMode
                ? t('navigation.editEnvironment')
                : t(`navigation.${currentNav.key}`)}
          </span>
        </>
      )}
    </div>
  );
}
