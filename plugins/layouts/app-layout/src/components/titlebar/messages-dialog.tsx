import { useEffect, useMemo } from 'react';
import {
  BellOff,
  CheckCheck,
  Inbox,
  Info,
  Loader2,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { format, isToday, isYesterday } from 'date-fns';
import { zhCN, enUS } from 'date-fns/locale';
import {
  ExtendedDialog,
  ExtendedDialogContent,
  ExtendedDialogTitle,
} from '@/components/extended-dialog';
import { Button } from '@/components/ui/button';
import { useMessagesStore } from '../../stores/messages-store';
import {
  NAV_TYPE_MAP,
  TEAM_MESSAGE_TYPES,
  MESSAGE_TYPE_ICONS,
  getMessageTypeName,
  getMessageTypeIconColor,
  isTeamMessageType,
} from './message-utils';
import { cn } from '@/lib/utils';

interface MessagesDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function MessagesDialog({ open, onOpenChange }: MessagesDialogProps) {
  const { t, i18n } = useTranslation('appLayout');

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

  useEffect(() => {
    if (open) {
      void refresh();
    }
  }, [open, refresh]);

  const unreadByType = useMemo(() => {
    const counts: Record<string, number> = {
      system_notification: 0,
      team: 0,
      private_chat: 0,
    };

    messages.forEach((msg) => {
      if (msg.is_read) return;

      if (isTeamMessageType(msg.message_type)) {
        counts.team += 1;
        return;
      }

      counts[msg.message_type] = (counts[msg.message_type] || 0) + 1;
    });

    return counts;
  }, [messages]);

  const navItems = [
    { key: null, icon: Inbox, label: t('notification.filterAll'), count: unreadCount },
    ...Object.keys(NAV_TYPE_MAP).map((type) => ({
      key: type,
      icon: MESSAGE_TYPE_ICONS[type] || Info,
      label: t(NAV_TYPE_MAP[type]),
      count: unreadByType[type] || 0,
    })),
  ];

  const readFilters = [
    { key: null, label: t('notification.filterAll') },
    { key: false, label: t('notification.filterUnread') },
    { key: true, label: t('notification.filterRead') },
  ];

  const formatTime = (dateString: string) => {
    try {
      const date = new Date(dateString);
      if (isToday(date)) return format(date, 'HH:mm', { locale: dateLocale });
      if (isYesterday(date)) return i18n.language === 'zh-CN' ? '昨天' : 'Yesterday';
      return format(date, 'MM/dd', { locale: dateLocale });
    } catch {
      return '';
    }
  };

  const handleMessageClick = async (messageUuid: string, isRead: boolean) => {
    if (!isRead) {
      await markAsRead(messageUuid);
    }
  };

  const handleTypeFilter = (type: string | null) => {
    if (type === 'team') {
      setMessageTypeFilter(TEAM_MESSAGE_TYPES[0]);
      return;
    }
    setMessageTypeFilter(type);
  };

  const isTeamFilterActive = messageTypeFilter !== null && isTeamMessageType(messageTypeFilter);

  return (
    <ExtendedDialog open={open} onOpenChange={onOpenChange}>
      <ExtendedDialogContent
        className="h-[calc(100vh-8rem)] w-[calc(100vw-2rem)] max-w-[1120px] gap-0 overflow-hidden border-0 bg-background p-0 sm:max-w-[1120px]"
        onPointerDownOutside={(event) => event.preventDefault()}
        closeButton={{
          className:
            'top-5 right-5 z-10 flex h-9 w-9 items-center justify-center rounded-full border-0 bg-transparent text-muted-foreground hover:bg-accent/50 hover:text-foreground',
        }}
      >
        <ExtendedDialogTitle className="sr-only">{t('notification.title')}</ExtendedDialogTitle>

        <div className="flex h-full min-h-0 flex-col">
          <header className="shrink-0 px-6 pb-4 pt-5">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-[11px] uppercase tracking-[0.16em] text-muted-foreground">
                  Message Center
                </p>
                <h2 className="mt-2 text-2xl font-semibold tracking-[-0.03em] text-foreground">
                  {t('notification.title')}
                </h2>
                <p className="mt-2 text-sm text-muted-foreground">
                  {unreadCount > 0
                    ? t('notification.unread', { count: unreadCount })
                    : t('notification.emptyDescription')}
                </p>
              </div>

              <div className="flex items-center gap-2 pr-10">
                {unreadCount > 0 ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 px-3 text-xs text-muted-foreground"
                    onClick={() => void markAllAsRead()}
                  >
                    <CheckCheck className="mr-1.5 h-3.5 w-3.5" />
                    {t('notification.markAllRead')}
                  </Button>
                ) : null}
              </div>
            </div>

            <nav className="mt-5 flex items-center gap-0 overflow-x-auto border-b border-border/60 pb-3">
              {navItems.map((item, index) => {
                const Icon = item.icon;
                const isActive =
                  item.key === null
                    ? messageTypeFilter === null
                    : item.key === 'team'
                      ? isTeamFilterActive
                      : messageTypeFilter === item.key;

                return (
                  <div key={item.key ?? 'all'} className="flex items-center">
                    <button
                      onClick={() => handleTypeFilter(item.key)}
                      className={cn(
                        'relative inline-flex min-h-10 items-center justify-center gap-2 whitespace-nowrap px-4 pb-3 text-sm font-medium transition-colors',
                        isActive
                          ? 'text-foreground'
                          : 'text-muted-foreground hover:text-foreground'
                      )}
                    >
                      <Icon className="h-4 w-4 shrink-0" />
                      <span>{item.label}</span>
                      {item.count > 0 ? (
                        <span
                          className={cn(
                            'inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] leading-5',
                            isActive ? 'bg-primary text-primary-foreground' : 'bg-accent text-muted-foreground'
                          )}
                        >
                          {item.count > 99 ? '99+' : item.count}
                        </span>
                      ) : null}
                      <span
                        className={cn(
                          'absolute bottom-0 left-4 right-4 h-[3px] rounded-full bg-primary transition-opacity',
                          isActive ? 'opacity-100' : 'opacity-0'
                        )}
                      />
                    </button>
                    {index < navItems.length - 1 ? (
                      <span className="mb-3 h-3.5 w-px bg-border/70" aria-hidden="true" />
                    ) : null}
                  </div>
                );
              })}
            </nav>

            <div className="mt-4 flex flex-wrap items-center gap-2">
              {readFilters.map((filter) => (
                <button
                  key={String(filter.key)}
                  onClick={() => setIsReadFilter(filter.key as boolean | null)}
                  className={cn(
                    'rounded-full px-3 py-1.5 text-xs transition-colors',
                    isReadFilter === filter.key
                      ? 'bg-foreground text-background'
                      : 'text-muted-foreground hover:bg-accent/60 hover:text-foreground'
                  )}
                >
                  {filter.label}
                </button>
              ))}
            </div>
          </header>

          <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-6">
            {loading && messages.length === 0 ? (
              <div className="flex h-full flex-col items-center justify-center text-center">
                <Loader2 className="mb-3 h-5 w-5 animate-spin text-muted-foreground" />
                <p className="text-sm text-muted-foreground">{t('notification.loading')}</p>
              </div>
            ) : error ? (
              <div className="flex h-full flex-col items-center justify-center text-center">
                <p className="text-sm text-destructive">{error}</p>
              </div>
            ) : messages.length === 0 ? (
              <div className="flex h-full flex-col items-center justify-center text-center">
                <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-muted/70">
                  <BellOff className="h-6 w-6 text-muted-foreground/40" />
                </div>
                <p className="text-sm text-foreground">{t('notification.empty')}</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  {t('notification.emptyDescription')}
                </p>
              </div>
            ) : (
              <div className="w-full">
                {messages.map((message, index) => {
                  const Icon = MESSAGE_TYPE_ICONS[message.message_type] || Info;
                  const iconColor = getMessageTypeIconColor(message.message_type);
                  const isUnread = !message.is_read;

                  return (
                    <div
                      key={message.message_uuid}
                      onClick={() => void handleMessageClick(message.message_uuid, message.is_read)}
                      className={cn(
                        'group cursor-pointer py-4 transition-colors',
                        index !== 0 && 'border-t border-border/50',
                        isUnread ? 'bg-primary/[0.03] hover:bg-primary/[0.05]' : 'hover:bg-accent/30'
                      )}
                    >
                      <div className="flex items-start gap-3">
                        <div
                          className={cn(
                            'mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl',
                            isUnread ? 'bg-primary/10' : 'bg-muted/60'
                          )}
                        >
                          <Icon className={cn('h-4 w-4', iconColor)} />
                        </div>

                        <div className="min-w-0 flex-1">
                          <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0">
                              <div className="flex items-center gap-2">
                                {isUnread ? (
                                  <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-primary" />
                                ) : null}
                                <h4
                                  className={cn(
                                    'truncate text-sm font-medium',
                                    isUnread ? 'text-foreground' : 'text-foreground/80'
                                  )}
                                >
                                  {message.title}
                                </h4>
                              </div>

                              <div className="mt-1 flex items-center gap-2 text-[11px] text-muted-foreground">
                                <span>{getMessageTypeName(message.message_type, t)}</span>
                                <span className="text-muted-foreground/40">·</span>
                                <time className="tabular-nums">
                                  {formatTime(message.message_created_at)}
                                </time>
                                {message.sender_name ? (
                                  <>
                                    <span className="text-muted-foreground/40">·</span>
                                    <span className="truncate">{message.sender_name}</span>
                                  </>
                                ) : null}
                              </div>
                            </div>

                            {isUnread ? (
                              <span className="mt-1 shrink-0 text-[10px] font-medium uppercase tracking-[0.14em] text-primary">
                                New
                              </span>
                            ) : null}
                          </div>

                          {message.content ? (
                            <p className="mt-2 text-sm leading-6 text-muted-foreground">
                              {message.content}
                            </p>
                          ) : null}

                        </div>
                      </div>
                    </div>
                  );
                })}

                {currentPage < totalPages ? (
                  <div className="border-t border-border/50 py-4">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-8 text-xs text-muted-foreground"
                      onClick={() => void loadMore()}
                      disabled={loading}
                    >
                      {loading ? (
                        <>
                          <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                          {t('notification.loading')}
                        </>
                      ) : (
                        t('notification.loadMore')
                      )}
                    </Button>
                  </div>
                ) : null}
              </div>
            )}
          </div>
        </div>
      </ExtendedDialogContent>
    </ExtendedDialog>
  );
}
