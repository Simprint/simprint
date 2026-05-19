import type { ElementType } from 'react';
import { useTranslation } from 'react-i18next';
import {
  MousePointer2,
  Keyboard,
  Clock,
  GitBranch,
  LogOut,
  PanelsTopLeft,
  Repeat,
  Camera,
  Navigation,
  FileText,
  Upload,
  Globe,
  Code,
} from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

export interface ComponentItem {
  id: string;
  type: string;
  name: string;
  icon: ElementType;
  category: 'browser' | 'flow' | 'data' | 'advanced';
  description: string;
}

const components: ComponentItem[] = [
  // 浏览器操作
  {
    id: 'click',
    type: 'click',
    name: '点击元素',
    icon: MousePointer2,
    category: 'browser',
    description: '点击页面上的元素',
  },
  {
    id: 'input',
    type: 'input',
    name: '输入文本',
    icon: Keyboard,
    category: 'browser',
    description: '在输入框中输入文本',
  },
  {
    id: 'navigate',
    type: 'navigate',
    name: '打开新页面',
    icon: Navigation,
    category: 'browser',
    description: '打开一个新的页面并访问指定地址',
  },
  {
    id: 'select-tab',
    type: 'select_tab',
    name: '选择标签页',
    icon: PanelsTopLeft,
    category: 'browser',
    description: '切换到指定位置的标签页',
  },
  {
    id: 'close-tab',
    type: 'close_tab',
    name: '关闭标签页',
    icon: PanelsTopLeft,
    category: 'browser',
    description: '关闭指定位置的标签页',
  },
  {
    id: 'screenshot',
    type: 'screenshot',
    name: '截图',
    icon: Camera,
    category: 'browser',
    description: '截取当前页面',
  },
  {
    id: 'scroll',
    type: 'scroll',
    name: '滚动页面',
    icon: Globe,
    category: 'browser',
    description: '滚动页面到指定位置',
  },
  // 流程控制
  {
    id: 'wait',
    type: 'wait',
    name: '等待',
    icon: Clock,
    category: 'flow',
    description: '等待指定时间或条件',
  },
  {
    id: 'condition',
    type: 'condition',
    name: '条件判断',
    icon: GitBranch,
    category: 'flow',
    description: '根据条件执行不同分支',
  },
  {
    id: 'loop',
    type: 'loop',
    name: '循环',
    icon: Repeat,
    category: 'flow',
    description: '重复执行一组操作',
  },
  {
    id: 'break-loop',
    type: 'break_loop',
    name: '退出循环',
    icon: LogOut,
    category: 'flow',
    description: '立即退出当前循环并继续后续流程',
  },
  // 数据操作
  {
    id: 'extract',
    type: 'extract',
    name: '提取页面数据',
    icon: FileText,
    category: 'data',
    description: '从页面元素中提取文本、链接或输入值',
  },
  {
    id: 'upload',
    type: 'upload',
    name: '上传文件',
    icon: Upload,
    category: 'data',
    description: '上传本地文件',
  },
  // 高级操作
  {
    id: 'script',
    type: 'script',
    name: '执行脚本',
    icon: Code,
    category: 'advanced',
    description: '执行自定义 JavaScript',
  },
];

interface ComponentPanelProps {
  selectedComponent: ComponentItem | null;
  onSelectComponent: (component: ComponentItem | null) => void;
}

export function ComponentPanel({ selectedComponent, onSelectComponent }: ComponentPanelProps) {
  const { t } = useTranslation('rpa');

  const categories = [
    { key: 'browser', label: t('editor.categories.browser') },
    { key: 'flow', label: t('editor.categories.flow') },
    { key: 'data', label: t('editor.categories.data') },
    { key: 'advanced', label: t('editor.categories.advanced') },
  ];

  const handleClick = (component: ComponentItem) => {
    // 如果点击的是已选中的组件，则取消选择
    if (selectedComponent?.id === component.id) {
      onSelectComponent(null);
    } else {
      onSelectComponent(component);
    }
  };

  return (
    <div className="w-56 border-r border-border bg-background flex flex-col min-h-0 overflow-hidden">
      <div className="px-3 py-2 border-b border-border shrink-0">
        <h3 className="text-xs font-semibold text-foreground">{t('editor.components')}</h3>
        {selectedComponent && (
          <p className="text-[10px] text-primary mt-1">{t('editor.clickCanvasToAdd')}</p>
        )}
      </div>
      <ScrollArea className="flex-1 min-h-0">
        <div className="p-2 space-y-4">
          {categories.map((category) => (
            <div key={category.key}>
              <div className="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {category.label}
              </div>
              <div className="space-y-1">
                {components
                  .filter((c) => c.category === category.key)
                  .map((component) => {
                    const isSelected = selectedComponent?.id === component.id;
                    return (
                      <div
                        key={component.id}
                        onClick={() => handleClick(component)}
                        className={`flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer transition-colors group ${
                          isSelected ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'
                        }`}
                        title={component.description}
                      >
                        <component.icon
                          className={`h-4 w-4 ${
                            isSelected
                              ? 'text-primary-foreground'
                              : 'text-muted-foreground group-hover:text-foreground'
                          }`}
                        />
                        <span
                          className={`text-xs ${
                            isSelected
                              ? 'text-primary-foreground'
                              : 'text-muted-foreground group-hover:text-foreground'
                          }`}
                        >
                          {component.name}
                        </span>
                      </div>
                    );
                  })}
              </div>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}



