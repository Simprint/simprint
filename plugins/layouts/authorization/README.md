# Authorization Layout Plugin

认证布局插件，提供本地用户选择、密码登录和本地用户创建页面的布局结构。

## 架构说明

本插件采用**布局插件 + 页面插件**的架构：

- **布局插件**（`plugins/layouts/authorization`）：提供布局结构
  - 左侧品牌区域（60%）：包含品牌 Logo、描述文字和 Canvas 装饰
  - 右侧表单区域（40%）：使用 `<Outlet />` 渲染子路由页面

- **页面插件**（`plugins/pages/`）：
  - `login`：本地用户选择与密码登录页面（路径：`/auth/login`）
  - `register`：本地用户创建页面（路径：`/auth/register`）

## 功能特性

- **布局结构**：3:2 比例的左右分栏布局
- **Canvas 装饰**：动态绘制的渐变圆形、线条和点阵装饰
- **响应式设计**：适配不同屏幕尺寸
- **网格背景纹理**：左侧面板的网格背景效果

## 组件结构

```
plugins/layouts/authorization/
├── src/
│   ├── index.tsx                    # 主布局组件（使用 Outlet 渲染子路由）
│   ├── components/
│   │   └── decorative-canvas.tsx   # Canvas 装饰组件
│   └── styles.css                   # 布局样式文件
├── manifest.json
├── package.json
└── tsconfig.json
```

## 路由配置

认证相关的路由都使用 `authorization-layout` 布局：

- `/auth/login` - 本地用户选择与密码登录页面
- `/auth/register` - 本地用户创建页面

路由系统会自动识别以 `/auth` 开头的路由，并使用 `authorization-layout` 布局。

## 样式说明

样式文件 `styles.css` 只包含布局相关的样式：

- 容器和面板布局
- 品牌区域样式
- 表单容器样式
- 响应式媒体查询

表单相关的样式（如输入框、按钮等）由各个页面插件自行管理。

## 开发规范

本插件遵循 Simprint 项目开发规范：

- 使用 TypeScript
- 使用 React Hooks
- 遵循项目代码风格
- 使用项目 CSS 变量系统
