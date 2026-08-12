/**
 * 用户信息接口
 */
export interface User {
  uuid: string;
  id: string;
  nickname?: string;
  avatar?: string;
  has_password?: boolean;
  status: string;
  current_workspace_uuid?: string | null; // 当前工作空间 UUID
  current_team_uuid?: string | null; // 当前团队 UUID
}

/**
 * 应用状态接口
 */
export interface AppState {
  /** 用户信息 */
  user: User | null;
  /** 是否已登录 */
  isAuthenticated: boolean;
  /** 是否正在加载 */
  isLoading: boolean;
}

/**
 * 状态管理服务契约
 */
export interface IStoreService {
  /**
   * 获取当前状态
   */
  getState(): AppState;

  /**
   * 订阅状态变化
   * @param callback 状态变化回调
   * @returns 取消订阅函数
   */
  subscribe(callback: (state: AppState) => void): () => void;

  /**
   * 设置用户信息（登录）
   * @param user 用户信息
   */
  setUser(user: User): void;

  /**
   * 清除用户信息（登出）
   */
  clearUser(): void;

  /**
   * 设置加载状态
   * @param isLoading 是否正在加载
   */
  setLoading(isLoading: boolean): void;
}
