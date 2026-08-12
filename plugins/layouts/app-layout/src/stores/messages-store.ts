import { create } from 'zustand';
import {
  listMessages,
  markMessageRead,
  batchMarkMessagesRead,
  getMessageStats,
  type Message,
  type MessageListRequest,
} from '../api/messages';
import { toast } from 'sonner';

const PAGE_SIZE = 20;

interface MessagesState {
  messages: Message[];
  stats: {
    total: number;
    unread: number;
    by_type: Record<string, number>;
  } | null;
  loading: boolean;
  error: string | null;
  currentPage: number;
  totalPages: number;
  messageTypeFilter: string | null;
  isReadFilter: boolean | null;
}

interface MessagesActions {
  loadMessages: (page?: number, append?: boolean) => Promise<void>;
  loadStats: () => Promise<void>;
  refresh: () => Promise<void>;
  loadMore: () => Promise<void>;
  markAsRead: (messageUuid: string) => Promise<void>;
  markAllAsRead: () => Promise<void>;
  setMessageTypeFilter: (type: string | null) => void;
  setIsReadFilter: (isRead: boolean | null) => void;
}

export const useMessagesStore = create<MessagesState & MessagesActions>((set, get) => ({
  // State
  messages: [],
  stats: null,
  loading: false,
  error: null,
  currentPage: 1,
  totalPages: 1,
  messageTypeFilter: null,
  isReadFilter: null,

  // Actions
  loadMessages: async (page = 1, append = false) => {
    set({ loading: true, error: null });

    try {
      const { messageTypeFilter, isReadFilter } = get();
      const request: MessageListRequest = {
        page,
        page_size: PAGE_SIZE,
        filters: {
          ...(messageTypeFilter && { message_type: messageTypeFilter }),
          ...(isReadFilter !== null && { is_read: isReadFilter }),
        },
      };

      const response = await listMessages(request);

      set((state) => ({
        messages: append ? [...state.messages, ...response.items] : response.items,
        currentPage: page,
        totalPages: Math.ceil(response.total / PAGE_SIZE),
        loading: false,
      }));
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : '加载消息失败';
      set({ error: errorMessage, loading: false });
      console.error('Failed to load messages:', e);
    }
  },

  loadStats: async () => {
    try {
      const statsData = await getMessageStats();
      set({ stats: statsData });
    } catch (e) {
      console.error('Failed to load message stats:', e);
    }
  },

  refresh: async () => {
    const { loadMessages, loadStats } = get();
    await Promise.all([loadMessages(1, false), loadStats()]);
  },

  loadMore: async () => {
    const { currentPage, totalPages, loading, loadMessages } = get();
    if (currentPage < totalPages && !loading) {
      await loadMessages(currentPage + 1, true);
    }
  },

  markAsRead: async (messageUuid: string) => {
    try {
      await markMessageRead(messageUuid);
      set((state) => ({
        messages: state.messages.map((msg) =>
          msg.message_uuid === messageUuid
            ? { ...msg, is_read: true, read_at: new Date().toISOString() }
            : msg
        ),
        stats: state.stats
          ? {
              ...state.stats,
              unread: Math.max(0, state.stats.unread - 1),
            }
          : null,
      }));
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '标记已读失败');
    }
  },

  markAllAsRead: async () => {
    const { messages } = get();
    const unreadMessages = messages.filter((msg) => !msg.is_read);
    if (unreadMessages.length === 0) return;

    try {
      const unreadUuids = unreadMessages.map((msg) => msg.message_uuid);
      await batchMarkMessagesRead(unreadUuids);

      set((state) => ({
        messages: state.messages.map((msg) => ({
          ...msg,
          is_read: true,
          read_at: msg.read_at || new Date().toISOString(),
        })),
        stats: state.stats
          ? {
              ...state.stats,
              unread: 0,
            }
          : null,
      }));

      toast.success(`已标记 ${unreadUuids.length} 条消息为已读`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : '批量标记已读失败');
    }
  },

  setMessageTypeFilter: (type: string | null) => {
    set({ messageTypeFilter: type });
    // 筛选条件改变时重新加载
    get().loadMessages(1, false);
  },

  setIsReadFilter: (isRead: boolean | null) => {
    set({ isReadFilter: isRead });
    // 筛选条件改变时重新加载
    get().loadMessages(1, false);
  },
}));
