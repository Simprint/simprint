import { useState, useEffect } from 'react';
import {
  Bell,
  CheckCheck,
  Filter,
  Loader2,
  Eye,
  Info,
  UserMinus,
  MessageSquare,
  AlertCircle,
  ChevronRight,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { formatDistanceToNow } from 'date-fns';
import { zhCN, enUS } from 'date-fns/locale';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { useMessagesStore } from '../../stores/messages-store';
import { MessagesDialog } from './messages-dialog';
import { cn } from '@/lib/utils';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

// 消息类型映射
const MESSAGE_TYPE_MAP: Record<string, string> = {
  system_notification: 'notification.systemNotification',
  team_removal: 'notification.teamRemoval',
  team_announcement: 'notification.teamAnnouncement',
  private_chat: 'notification.privateChat',
};

// 消息类型图标映射
const MESSAGE_TYPE_ICONS: Record<string, typeof Info> = {
  system_notification: Info,
  team_removal: UserMinus,
  team_announcement: MessageSquare,
  private_chat: MessageSquare,
};

/**
 * 通知菜单组件
 */
export function NotificationMenu() {
  const { t, i18n } = useTranslation('appLayout');
  const [open, setOpen] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);

  // 使用全局消息store
  const messages = useMessagesStore((state) => state.messages);
  const stats = useMessagesStore((state) => state.stats);
  const loading = useMessagesStore((state) => state.loading);
  const error = useMessagesStore((state) => state.error);
  const currentPage = useMessagesStore((state) => state.currentPage);
  const totalPages = useMessagesStore((state) => state.totalPages);
  const messageTypeFilter = useMessagesStore((state) => state.messageTypeFilter);
  const isReadFilter = useMessagesStore((state) => state.isReadFilter);
  const refresh = useMessagesStore((state) => state.refresh);
  const loadMore = useMessagesStore((state) => state.loadMore);
  const markAsRead = useMessagesStore((state) => state.markAsRead);
  const markAllAsRead = useMessagesStore((state) => state.markAllAsRead);
  const setMessageTypeFilter = useMessagesStore((state) => state.setMessageTypeFilter);
  const setIsReadFilter = useMessagesStore((state) => state.setIsReadFilter);

  const unreadCount = stats?.unread || 0;
  const dateLocale = i18n.language === 'zh-CN' ? zhCN : enUS;

  // 当菜单打开时刷新消息
  useEffect(() => {
    if (open) {
      void refresh();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  // 格式化时间
  const formatTime = (dateString: string) => {
    try {
      return formatDistanceToNow(new Date(dateString), {
        addSuffix: true,
        locale: dateLocale,
      });
    } catch {
      return dateString;
    }
  };

  // 获取消息类型显示名称
  const getMessageTypeName = (type: string) => {
    const key = MESSAGE_TYPE_MAP[type] || type;
    return t(key);
  };

  // 获取消息类型图标
  const getMessageTypeIcon = (type: string) => {
    return MESSAGE_TYPE_ICONS[type] || Info;
  };

  // 获取消息类型图标颜色
  const getMessageTypeIconColor = (type: string) => {
    const colorMap: Record<string, string> = {
      system_notification: 'text-blue-500',
      team_removal: 'text-red-500',
      team_announcement: 'text-purple-500',
      private_chat: 'text-orange-500',
    };
    return colorMap[type] || 'text-muted-foreground';
  };

  // 处理消息点击（标记为已读）
  const handleMessageClick = async (messageUuid: string, isRead: boolean) => {
    if (!isRead) {
      await markAsRead(messageUuid);
    }
  };

  // 获取已读状态显示文本
  const getReadStatusText = () => {
    if (isReadFilter === null) return t('notification.filterAll');
    if (isReadFilter === false) return t('notification.filterUnread');
    return t('notification.filterRead');
  };

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <button
          className="relative w-8 h-8 flex items-center justify-center text-muted-foreground/80 hover:bg-accent/80 hover:text-foreground transition-all duration-200 ease-in-out rounded-sm cursor-pointer outline-none"
          title={t('notification.title')}
        >
          <Bell className="w-4 h-4" />
          {unreadCount > 0 && (
            <span className="absolute -top-0.5 -right-0.5 w-2 h-2 bg-destructive rounded-full" />
          )}
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-96 max-h-[600px] overflow-hidden flex flex-col">
        {/* 头部 */}
        <div className="flex items-center justify-between px-4 py-3 border-b bg-muted/30">
          <DropdownMenuLabel className="p-0">
            {t('notification.title')}
            {unreadCount > 0 && (
              <span className="ml-2 text-xs text-muted-foreground font-normal">
                ({unreadCount})
              </span>
            )}
          </DropdownMenuLabel>
          <div className="flex items-center gap-2">
            {/* 已读状态筛选器（下拉菜单） */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  className="h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
                  title={getReadStatusText()}
                >
                  <Eye className="w-3.5 h-3.5" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-32">
                <DropdownMenuItem
                  onClick={() => setIsReadFilter(null)}
                  className={cn('cursor-pointer', isReadFilter === null && 'bg-accent')}
                >
                  {t('notification.filterAll')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => setIsReadFilter(false)}
                  className={cn('cursor-pointer', isReadFilter === false && 'bg-accent')}
                >
                  {t('notification.filterUnread')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => setIsReadFilter(true)}
                  className={cn('cursor-pointer', isReadFilter === true && 'bg-accent')}
                >
                  {t('notification.filterRead')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* 消息类型筛选器（图标按钮） */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  className="h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
                  title={t('notification.filterType')}
                >
                  <Filter className="w-3.5 h-3.5" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-40">
                <DropdownMenuItem
                  onClick={() => setMessageTypeFilter(null)}
                  className={cn('cursor-pointer', messageTypeFilter === null && 'bg-accent')}
                >
                  {t('notification.filterAll')}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                {Object.keys(MESSAGE_TYPE_MAP).map((type) => (
                  <DropdownMenuItem
                    key={type}
                    onClick={() => setMessageTypeFilter(type)}
                    className={cn('cursor-pointer', messageTypeFilter === type && 'bg-accent')}
                  >
                    {getMessageTypeName(type)}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* 全部标记为已读 */}
            {unreadCount > 0 && (
              <Button
                variant="ghost"
                size="sm"
                className="h-7 w-7 p-0"
                onClick={(e) => {
                  e.stopPropagation();
                  markAllAsRead();
                }}
                title={t('notification.markAllRead')}
              >
                <CheckCheck className="w-3.5 h-3.5" />
              </Button>
            )}
          </div>
        </div>

        {/* 消息列表 */}
        <div className="flex-1 overflow-y-auto">
          {loading && messages.length === 0 ? (
            <div className="py-8 text-center text-sm text-muted-foreground flex items-center justify-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin" />
              {t('notification.loading')}
            </div>
          ) : error ? (
            <div className="py-8 text-center text-sm text-destructive">{error}</div>
          ) : messages.length === 0 ? (
            <div className="py-8 text-center text-sm text-muted-foreground">
              {t('notification.empty')}
            </div>
          ) : (
            <div className="py-2 space-y-1">
              {messages.map((message, index) => {
                const MessageIcon = getMessageTypeIcon(message.message_type);
                const iconColor = getMessageTypeIconColor(message.message_type);
                const isUnread = !message.is_read;

                return (
                  <div
                    key={message.message_uuid}
                    className={cn(
                      'group relative mx-2 rounded-lg border transition-all duration-200 cursor-pointer',
                      isUnread
                        ? 'bg-accent/30 border-primary/20 hover:bg-accent/50 hover:border-primary/40 hover:shadow-md hover:shadow-primary/5'
                        : 'bg-transparent border-transparent hover:bg-accent/30 hover:border-border/50'
                    )}
                    onClick={() => handleMessageClick(message.message_uuid, message.is_read)}
                  >
                    {/* 未读消息左侧指示条 */}
                    {isUnread && (
                      <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary rounded-l-lg" />
                    )}

                    <div className="flex items-start gap-3 px-3 py-2.5">
                      {/* 消息类型图标容器 */}
                      <div
                        className={cn(
                          'shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-all duration-200',
                          isUnread
                            ? 'bg-primary/10 border border-primary/20 group-hover:bg-primary/15 group-hover:border-primary/30'
                            : 'bg-muted/50 border border-border/50 group-hover:bg-muted group-hover:border-border'
                        )}
                      >
                        <MessageIcon className={cn('w-4 h-4', iconColor)} />
                      </div>

                      {/* 消息内容 */}
                      <div className="flex-1 min-w-0 space-y-1.5">
                        {/* 标题行 */}
                        <div className="flex items-start justify-between gap-2">
                          <p
                            className={cn(
                              'text-sm font-semibold leading-snug transition-colors',
                              isUnread
                                ? 'text-foreground'
                                : 'text-muted-foreground group-hover:text-foreground'
                            )}
                          >
                            {message.title}
                          </p>
                          {isUnread && (
                            <span className="w-2 h-2 bg-primary rounded-full shrink-0 mt-1.5 animate-pulse" />
                          )}
                        </div>

                        {/* 内容预览 */}
                        {message.content && (
                          <p
                            className={cn(
                              'text-xs leading-relaxed line-clamp-2 transition-colors',
                              isUnread
                                ? 'text-muted-foreground'
                                : 'text-muted-foreground/70 group-hover:text-muted-foreground'
                            )}
                          >
                            {message.content}
                          </p>
                        )}

                        {/* 底部元信息 */}
                        <div className="flex items-center justify-between gap-2 pt-0.5">
                          <div className="flex items-center gap-1.5 text-xs">
                            {message.sender_name && (
                              <>
                                <span
                                  className={cn(
                                    'truncate max-w-[100px] font-medium transition-colors',
                                    isUnread
                                      ? 'text-muted-foreground'
                                      : 'text-muted-foreground/60 group-hover:text-muted-foreground'
                                  )}
                                >
                                  {message.sender_name}
                                </span>
                                <span className="text-muted-foreground/40">·</span>
                              </>
                            )}
                            <span
                              className={cn(
                                'truncate transition-colors',
                                isUnread
                                  ? 'text-muted-foreground'
                                  : 'text-muted-foreground/60 group-hover:text-muted-foreground'
                              )}
                            >
                              {formatTime(message.message_created_at)}
                            </span>
                          </div>
                          {message.priority === 'high' || message.priority === 'urgent' ? (
                            <div
                              className={cn(
                                'shrink-0 p-1 rounded',
                                message.priority === 'urgent'
                                  ? 'bg-destructive/10 text-destructive'
                                  : 'bg-orange-500/10 text-orange-500'
                              )}
                            >
                              <AlertCircle className="w-3 h-3" />
                            </div>
                          ) : null}
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
              {currentPage < totalPages && (
                <div className="px-4 py-3 border-t bg-muted/20">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="w-full text-xs h-8"
                    onClick={loadMore}
                    disabled={loading}
                  >
                    {loading ? (
                      <>
                        <Loader2 className="w-3 h-3 mr-1.5 animate-spin" />
                        {t('notification.loading')}
                      </>
                    ) : (
                      t('notification.loadMore')
                    )}
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              className="absolute bottom-0 -right-0 z-20 flex items-center justify-center text-sidebar-foreground/70 hover:text-foreground transition-colors duration-200 animate-[pulse-gentle_2s_ease-in-out_infinite] group cursor-pointer"
              onClick={(e) => {
                e.stopPropagation();
                setDialogOpen(true);
              }}
            >
              <ChevronRight className="rotate-45 transition-transform duration-200 opacity-80 group-hover:opacity-100 animate-[slide-hint_2s_ease-in-out_infinite]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="right">{t('notification.openDialog')}</TooltipContent>
        </Tooltip>
      </DropdownMenuContent>

      {/* 消息对话框 */}
      <MessagesDialog open={dialogOpen} onOpenChange={setDialogOpen} />
    </DropdownMenu>
  );
}
