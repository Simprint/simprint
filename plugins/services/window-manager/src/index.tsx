import { WindowManagerService } from './window-manager-service';

/**
 * Window Manager 服务插件
 * 注意：这是一个特殊的服务插件，它不遵循标准的插件格式
 * 因为它需要在应用启动时就被使用，但不渲染任何 UI
 */
const windowManagerPlugin = {
  id: 'window-manager',
  name: 'Window Manager Service',
  version: '1.0.0',
  component: WindowManagerService,
  slots: [],
};

export default windowManagerPlugin;
