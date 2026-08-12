import { Outlet } from 'react-router';
import { DecorativeCanvas } from './components/decorative-canvas';
import AuthorizationTitlebar from './components/titlebar';
import { extensionRegistry } from '@slotkitjs/core';
import { authLayoutResources } from './i18n/resources';
import { useTranslation } from 'react-i18next';

const AuthorizationLayoutPlugin: React.FC = () => {
  const { t } = useTranslation('authLayout');
  return (
    <div className="flex flex-col h-screen w-full overflow-hidden">
      {/* Titlebar */}
      <AuthorizationTitlebar />

      {/* 内容区域 */}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        {/* 左侧品牌区域 */}
        <div className="relative hidden flex-col items-start justify-end overflow-hidden border-r border-border bg-secondary before:pointer-events-none before:absolute before:inset-0 before:z-1 before:bg-[linear-gradient(rgba(0,0,0,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(0,0,0,0.02)_1px,transparent_1px)] before:bg-size-[24px_24px] before:opacity-50 lg:flex lg:flex-[0_0_55%] lg:p-[60px_50px] xl:p-[40px_60px]">
          <DecorativeCanvas />
          <div className="relative z-2 w-full max-w-[700px] h-auto flex flex-col justify-end items-start">
            <div className="text-left mb-0">
              <div className="mb-2 text-[48px] font-semibold leading-none tracking-[-1px] text-foreground xl:text-[64px]">
                Simprint
              </div>
            </div>
            <div className="mt-10 max-w-[500px] lg:mt-[30px] md:mt-6">
              <div className="text-[32px] font-normal text-foreground mb-4 leading-[1.4] tracking-[-0.5px]">
                {t('brand.sloganTitle')}
              </div>
              <div className="text-base text-muted-foreground leading-[1.6] font-normal">
                {t('brand.sloganDesc')}
              </div>
            </div>
          </div>
        </div>

        {/* 右侧表单区域 */}
        <div className="relative flex min-w-0 flex-1 flex-col items-center justify-center overflow-y-auto bg-background px-6 py-8 sm:px-10 lg:flex-[0_0_45%] lg:p-[60px_40px]">
          <div className="w-full max-w-[400px] [&_h1]:text-[28px] [&_h1]:font-medium! [&_h1]:leading-[1.3] [&_h1]:tracking-[-0.3px] [&_h1]:text-foreground [&_p]:text-sm [&_p]:font-normal [&_p]:leading-normal [&_p]:text-muted-foreground [&_label]:text-sm [&_label]:font-normal [&_label]:leading-normal [&_label]:text-foreground [&_input]:text-sm [&_input]:font-normal [&_input]:leading-normal [&_input]:text-foreground [&_button]:text-sm [&_button]:font-normal [&_button]:leading-normal [&_.text-sm]:text-[13px] [&_.text-sm]:font-normal [&_.text-sm]:leading-normal [&_.text-destructive]:text-[13px] [&_.text-destructive]:font-normal [&_.text-destructive]:leading-normal **:font-normal!">
            <Outlet />
          </div>
        </div>
      </div>
    </div>
  );
};

// i18n resources
try {
  extensionRegistry.contribute('i18n:resources', {
    contributorId: 'authorization-layout',
    value: {
      namespace: 'authLayout',
      resources: authLayoutResources,
    },
    priority: 10,
  });
} catch (error) {
  console.warn('[authorization-layout] Failed to contribute i18n resources:', error);
}

const authorizationLayoutPlugin = {
  id: 'authorization-layout',
  name: 'Authorization Layout',
  version: '1.0.0',
  component: AuthorizationLayoutPlugin,
  slots: [],
};

export default authorizationLayoutPlugin;
