import { useEffect, useState } from 'react';
import { useRoutes } from 'react-router';
import { extensionRegistry, pluginRegistry } from '@slotkitjs/core';
import { useRouteConfig } from '../../hooks/useRouteConfig';
import type { RouteConfig } from '../../types';

interface AppRoutesProps {
  onReady?: () => void;
}

/**
 * 应用路由组件
 * 负责动态加载和管理路由
 */
export const AppRoutes: React.FC<AppRoutesProps> = ({ onReady }) => {
  const [routes, setRoutes] = useState<RouteConfig[]>([]);

  useEffect(() => {
    const checkLayout = () => {
      const layoutPlugin = pluginRegistry.getPlugin('app-layout');
      // 检查布局插件是否已加载（用于调试）
      if (layoutPlugin?.component) {
        console.log('[AppRoutes] Layout plugin loaded');
      }
    };

    const updateRoutes = () => {
      const routeContributions = extensionRegistry.getContributions<RouteConfig>('routes');
      const newRoutes = routeContributions.map((c) => c.value);
      console.log(
        '[AppRoutes] Routes updated:',
        newRoutes.map((r) => r.path)
      );
      setRoutes(newRoutes);
    };

    checkLayout();
    updateRoutes();

    // 订阅插件注册事件
    const unsubscribe = pluginRegistry.subscribe((event) => {
      if (event.type === 'register') {
        if (event.plugin?.id === 'app-layout') {
          console.log('[AppRoutes] Layout plugin registered');
        }
        // 当插件注册时，更新路由（插件可能在模块加载时已贡献路由）
        updateRoutes();
        checkLayout();
      }
    });

    return () => {
      unsubscribe();
    };
  }, []);

  const routeElements = useRouteConfig(routes);

  useEffect(() => {
    if (routes.length === 0 || !onReady) return;

    let secondFrame = 0;
    const firstFrame = requestAnimationFrame(() => {
      secondFrame = requestAnimationFrame(onReady);
    });

    return () => {
      cancelAnimationFrame(firstFrame);
      if (secondFrame) cancelAnimationFrame(secondFrame);
    };
  }, [routes, routeElements, onReady]);

  return useRoutes(routeElements);
};
