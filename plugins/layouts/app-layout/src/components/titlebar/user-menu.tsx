import { LogOut, ChevronDown, Palette, Languages, Users, Check, Building2 } from 'lucide-react';
import { useNavigate } from 'react-router';
import {
  useAuth,
  useSettingsDialogStore,
  useRefreshStore,
  useAuthStore,
  setGeneralSettings,
  type ThemeMode,
  type SettingsDialogTab,
} from '../../../../../services/store/src';
import { useTheme } from '@/components/theme-provider';
import { useTranslation } from 'react-i18next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from '@/components/ui/dropdown-menu';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { useUserMenuConfig } from './hooks/use-user-menu-config';
import { getMyTeams, switchTeam, type TeamItem } from '../../api/teams';
import { useWorkspaceStore } from '../../../../../services/store/src/stores/workspace';
import { WorkspaceSwitchDialog } from './workspace-switch-dialog';
import { useState, useEffect } from 'react';
import { toast } from 'sonner';

/**
 * 用户菜单组件
 */
export function UserMenu() {
  const { user, isAuthenticated, clearUser } = useAuth();
  const { theme, setTheme } = useTheme();
  const { i18n, t: tCommon } = useTranslation('common');
  const { t } = useTranslation('appLayout');
  const navigate = useNavigate();
  const { open: openSettingsDialog } = useSettingsDialogStore();
  const { themeOptions, languageOptions, settingsOptions } = useUserMenuConfig(t, tCommon);

  // 团队相关状态
  const [teams, setTeams] = useState<TeamItem[]>([]);
  const [currentTeamUuid, setCurrentTeamUuid] = useState<string | null>(null);
  const [loadingTeams, setLoadingTeams] = useState(false);

  // 工作空间相关状态（只读取，不主动加载）
  const { workspaces } = useWorkspaceStore();
  const [workspaceDialogOpen, setWorkspaceDialogOpen] = useState(false);

  // 监听团队数据刷新标识
  const teamsRefreshKey = useRefreshStore((state) => state.teams);
  const { setCurrentTeam } = useAuthStore();

  const handleLogout = async () => {
    // 清理当前本地会话，返回用户选择界面。
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('logout');
    } catch (e) {
      // 即使失败也继续清理前端状态并跳转，避免卡死在当前页面
      console.warn('[UserMenu] 退出登录时调用 tauri logout 失败:', e);
    }

    clearUser();
    navigate('/auth/login');
  };

  const handleSettingsOpen = (tabId: string) => {
    openSettingsDialog(tabId as SettingsDialogTab);
  };

  const handleLanguageChange = (lng: 'zh-CN' | 'en-US') => {
    void i18n.changeLanguage(lng);
    localStorage.setItem('simprint-ui-locale', lng);
    void setGeneralSettings({ language: lng });
  };

  const handleThemeChange = (next: ThemeMode) => {
    setTheme(next);
    void setGeneralSettings({ theme: next });
  };

  // 加载团队列表（当 teamsRefreshKey 变化时也会重新加载）
  useEffect(() => {
    const loadTeams = async () => {
      if (!isAuthenticated || !user) return;
      setLoadingTeams(true);
      try {
        const response = await getMyTeams();
        setTeams(response.teams);
        setCurrentTeamUuid(response.current_team_uuid);
      } catch (e) {
        // 静默失败，不显示错误提示
        console.error('Failed to load teams:', e);
      } finally {
        setLoadingTeams(false);
      }
    };
    void loadTeams();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAuthenticated, user?.uuid, teamsRefreshKey]);

  // 切换团队
  const handleTeamChange = async (teamUuid: string) => {
    if (teamUuid === currentTeamUuid) return;
    try {
      await switchTeam({ team_uuid: teamUuid });
      setCurrentTeamUuid(teamUuid);
      setCurrentTeam(teamUuid);
      // 更新 teams 列表中的 is_current 状态
      setTeams((prev) =>
        prev.map((team) => ({
          ...team,
          is_current: team.uuid === teamUuid,
        }))
      );
      toast.success(t('status.teamSwitchSuccess') || '切换团队成功');
      // 团队切换完成，其他组件可以通过监听团队状态变化来刷新数据
    } catch (e) {
      toast.error(e instanceof Error ? e.message : t('status.teamSwitchFailed') || '切换团队失败');
    }
  };

  const displayName = user?.nickname || t('status.user');

  // 获取当前工作空间
  const currentWorkspace = workspaces.find((ws) => ws.is_current);

  if (!isAuthenticated || !user) {
    return null;
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="flex items-center gap-2 px-5 h-8 text-[10px] text-foreground bg-muted hover:bg-primary/10 hover:text-foreground transition-all cursor-pointer outline-none rounded-sm mx-2">
            <Avatar className="w-6 h-6">
              <AvatarFallback className="bg-primary/10 text-primary text-xs font-semibold">
                {user.avatar || displayName.charAt(0).toUpperCase()}
              </AvatarFallback>
            </Avatar>
            <span className="font-mono tracking-tight text-[12px]">{displayName}</span>
            <ChevronDown className="w-3 h-3 text-muted-foreground" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56 overflow-y-hidden">
          <DropdownMenuLabel>
            <div className="flex flex-col space-y-1">
              <p className="text-sm font-medium leading-none">
                {displayName}
              </p>
              <p className="text-xs leading-none text-muted-foreground">{user.avatar}</p>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          {/* 系统设置选项 */}
          {settingsOptions.map((option) => {
            const Icon = option.icon;
            return (
              <DropdownMenuItem
                key={option.id}
                onClick={() => handleSettingsOpen(option.id)}
                className="cursor-pointer"
              >
                <Icon className="w-4 h-4" />
                <span>{option.label}</span>
              </DropdownMenuItem>
            );
          })}
          <DropdownMenuSeparator />
          {/* 工作空间 */}
          <DropdownMenuItem onClick={() => setWorkspaceDialogOpen(true)} className="cursor-pointer">
            <Building2 className="w-4 h-4" />
            <span>{t('status.workspace') || '工作空间'}</span>
            {currentWorkspace && (
              <span className="ml-auto text-xs text-muted-foreground truncate max-w-[100px]">
                {currentWorkspace.name}
              </span>
            )}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {/* 团队 */}
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="cursor-pointer">
              <Users className="w-4 h-4" />
              <span>{t('status.team')}</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              {loadingTeams ? (
                <DropdownMenuItem disabled className="cursor-default">
                  <span className="text-xs text-muted-foreground">
                    {t('status.loadingTeams') || '加载中...'}
                  </span>
                </DropdownMenuItem>
              ) : teams.length === 0 ? (
                <DropdownMenuItem disabled className="cursor-default">
                  <span className="text-xs text-muted-foreground">
                    {t('status.noTeams') || '暂无团队'}
                  </span>
                </DropdownMenuItem>
              ) : (
                teams.map((team) => {
                  const isActive = team.is_current;
                  return (
                    <DropdownMenuItem
                      key={team.uuid}
                      onClick={() => handleTeamChange(team.uuid)}
                      className="cursor-pointer"
                    >
                      <div className="flex items-center gap-2 w-full">
                        <span className="w-5 h-5 rounded-full bg-primary/15 text-primary text-[10px] font-semibold flex items-center justify-center shrink-0">
                          {team.name.charAt(0).toUpperCase()}
                        </span>
                        <span
                          className={`flex-1 truncate ${isActive ? 'text-primary font-medium' : ''}`}
                        >
                          {team.name}
                        </span>
                        {isActive && <Check className="w-4 h-4 text-primary shrink-0" />}
                      </div>
                    </DropdownMenuItem>
                  );
                })
              )}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator />
          {/* 语言 */}
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="cursor-pointer">
              <Languages className="w-4 h-4" />
              <span>{tCommon('language.title')}</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              {languageOptions.map((option) => {
                const isActive = i18n.language === option.id;
                return (
                  <DropdownMenuItem
                    key={option.id}
                    onClick={() => handleLanguageChange(option.id)}
                    className="cursor-pointer"
                  >
                    <option.icon className={`w-4 h-4 ${isActive ? 'text-primary' : ''}`} />
                    <span className={isActive ? 'text-primary font-medium' : ''}>
                      {option.label}
                    </span>
                  </DropdownMenuItem>
                );
              })}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator />
          {/* 主题 */}
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="cursor-pointer">
              <Palette className="w-4 h-4" />
              <span>{t('status.theme')}</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              {themeOptions.map((option) => {
                const Icon = option.icon;
                const isActive = theme === option.id;
                return (
                  <DropdownMenuItem
                    key={option.id}
                    onClick={() => handleThemeChange(option.id)}
                    className="cursor-pointer"
                  >
                    <Icon className={`w-4 h-4 ${isActive ? 'text-primary' : ''}`} />
                    <span className={isActive ? 'text-primary font-medium' : ''}>
                      {option.label}
                    </span>
                  </DropdownMenuItem>
                );
              })}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator />
          <DropdownMenuItem variant="destructive" onClick={handleLogout} className="cursor-pointer">
            <LogOut className="w-4 h-4" />
            <span>{t('status.logout')}</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <WorkspaceSwitchDialog open={workspaceDialogOpen} onOpenChange={setWorkspaceDialogOpen} />
    </>
  );
}
