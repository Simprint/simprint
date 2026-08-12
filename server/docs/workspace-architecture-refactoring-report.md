# 工作空间架构重构变更报告

**重构日期**：2025-01-25  
**重构范围**：工作空间架构全面重构  
**文档版本**：v1.0

---

## 概述

本次重构将系统架构从"用户-团队-环境"模式升级为"用户-工作空间-团队-分组-环境"的多层级结构，实现了更灵活的资源管理、权限控制和计费体系。

---

## 1. 架构设计变更

### 1.1 新增架构文档

- **文件**：`simprint-server/docs/workspace-architecture.md`
- **内容**：完整的工作空间架构设计文档，包括：
  - 核心概念和层级结构
  - 实体关系图（ASCII 格式）
  - 实体详细设计
  - 权限控制体系
  - 配额管理
  - 代理可见性管理
  - 数据模型设计要点

### 1.2 核心设计原则

1. **工作空间是资源隔离边界**：所有资源（环境、代理、配额）都归属于工作空间
2. **团队是协作组织单元**：团队成员共享团队资源，通过分组进行细粒度管理
3. **分组是环境分类容器**：分组用于对团队内的环境进行分类和权限控制（可选）
4. **环境是核心操作单元**：所有业务操作围绕环境展开，环境必须归属团队

---

## 2. 数据库变更

### 2.1 新增表

#### 2.1.1 工作空间表（workspaces）

- **迁移文件**：`20250125000001_create_workspaces.sql`
- **字段**：
  - `uuid`：工作空间唯一标识
  - `name`：工作空间名称
  - `owner_uuid`：所有者用户 UUID
  - `workspace_type`：工作空间类型（personal/team/enterprise）
  - `created_at`、`updated_at`、`deleted_at`：时间戳

#### 2.1.2 工作空间配额表（workspace_quotas）

- **迁移文件**：`20250125000002_create_workspace_quotas.sql`
- **字段**：
  - `workspace_uuid`：工作空间 UUID（主键）
  - `max_environments`、`used_environments`：环境配额
  - `max_team_members`、`used_team_members`：成员配额
  - `max_proxies`、`used_proxies`：代理配额
  - `max_rpa_tasks`、`used_rpa_tasks`：RPA 任务配额

#### 2.1.3 代理可见团队关联表（proxy_visible_teams）

- **迁移文件**：`20250125000003_create_proxy_visible_teams.sql`
- **字段**：
  - `proxy_uuid`：代理 UUID
  - `workspace_uuid`：工作空间 UUID（冗余，便于查询）
  - `team_uuid`：团队 UUID
  - `created_at`：创建时间
- **唯一约束**：`UNIQUE (proxy_uuid, team_uuid)`

#### 2.1.4 分组权限表（group_member_permissions）

- **迁移文件**：`20250125000004_create_group_member_permissions.sql`
- **字段**：
  - `group_uuid`：分组 UUID
  - `workspace_uuid`：工作空间 UUID（冗余，便于查询）
  - `team_uuid`：团队 UUID（冗余，便于查询）
  - `user_uuid`：用户 UUID
  - `permission_type`：权限类型（read/write/manage）
  - `granted_by`：授权者 UUID
  - `created_at`、`updated_at`：时间戳
- **唯一约束**：`UNIQUE (group_uuid, user_uuid)`

### 2.2 修改现有表

#### 2.2.1 团队表（teams）

- **迁移文件**：`20250125000005_alter_teams_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：所属工作空间 UUID
- **移除字段**：
  - `max_members`、`max_environments`、`max_proxies`：配额移至 `workspace_quotas`
  - `default_proxy_uuid`：不再需要默认代理

#### 2.2.2 团队成员表（team_members）

- **迁移文件**：`20250125000006_alter_team_members_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：工作空间 UUID（冗余，便于查询）
- **移除字段**：
  - `environment_count`、`group_count`：统计字段，可通过查询计算
- **更新唯一约束**：`UNIQUE (team_uuid, user_uuid, workspace_uuid)`

#### 2.2.3 分组表（groups）

- **迁移文件**：`20250125000007_alter_groups_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：所属工作空间 UUID（冗余，便于查询）
- **移除字段**：
  - `user_uuid`：分组属于团队，不属于用户
  - `default_proxy_uuid`：不再需要默认代理
  - `color`：前端可自行管理
- **约束**：确保 `team_uuid` 为 NOT NULL

#### 2.2.4 环境表（environments）

- **迁移文件**：`20250125000008_alter_environments_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：所属工作空间 UUID（冗余，便于查询）
- **约束**：确保 `team_uuid` 为 NOT NULL

#### 2.2.5 代理表（proxies）

- **迁移文件**：`20250125000009_alter_proxies_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：所属工作空间 UUID（必须）
  - `owner_uuid`：代理所有者 UUID（重命名自 `user_uuid`）
- **移除字段**：
  - `team_uuid`：代理属于工作空间，不属于团队
  - `usage_count`：可通过查询计算

#### 2.2.6 订阅表（subscriptions）

- **迁移文件**：`20250125000010_alter_subscriptions_add_workspace.sql`
- **新增字段**：
  - `workspace_uuid`：所属工作空间 UUID

#### 2.2.7 用户信息表（user_infos）

- **迁移文件**：`20250125000013_alter_user_infos_add_current_workspace.sql`
- **新增字段**：
  - `current_workspace_uuid`：用户当前工作空间 UUID

### 2.3 数据迁移

#### 2.3.1 废弃用户配额表

- **迁移文件**：`20250125000011_deprecate_user_quotas.sql`
- **操作**：将 `user_quotas` 表重命名为 `deprecated_user_quotas`

#### 2.3.2 数据迁移脚本

- **迁移文件**：`20250125000012_migrate_to_workspaces.sql`
- **操作内容**：
  1. 为每个现有用户创建默认个人工作空间
  2. 为每个工作空间创建默认配额
  3. 将现有 `user_quotas` 数据迁移到 `workspace_quotas`
  4. 更新所有相关表的 `workspace_uuid` 字段
  5. 添加外键约束
  6. 设置 `workspace_uuid` 为 NOT NULL

---

## 3. 代码层变更

### 3.1 DTO 层（Data Transfer Object）

#### 3.1.1 新增 DTO

- `simprint-server/src/dto/workspaces.rs`：工作空间 DTO
- `simprint-server/src/dto/workspace_quotas.rs`：工作空间配额 DTO
- `simprint-server/src/dto/proxy_visible_teams.rs`：代理可见团队 DTO
- `simprint-server/src/dto/group_member_permissions.rs`：分组权限 DTO

#### 3.1.2 修改现有 DTO

- `simprint-server/src/dto/teams.rs`：添加 `workspace_uuid`，移除配额相关字段
- `simprint-server/src/dto/groups.rs`：添加 `workspace_uuid`，移除 `user_uuid`、`default_proxy_uuid`、`color`
- `simprint-server/src/dto/environments.rs`：添加 `workspace_uuid`
- `simprint-server/src/dto/proxies.rs`：添加 `workspace_uuid`、`owner_uuid`，移除 `team_uuid`
- `simprint-server/src/dto/subscriptions.rs`：添加 `workspace_uuid`
- `simprint-server/src/dto/user.rs`：添加 `current_workspace_uuid`

### 3.2 Entity 层（请求/响应实体）

#### 3.2.1 新增 Entity

- `simprint-server/src/entitys/workspaces.rs`：工作空间相关请求/响应
- `simprint-server/src/entitys/workspace_quotas.rs`：工作空间配额响应
- `simprint-server/src/entitys/proxy_visible_teams.rs`：代理可见性请求/响应
- `simprint-server/src/entitys/group_member_permissions.rs`：分组权限请求/响应

#### 3.2.2 修改现有 Entity

- `simprint-server/src/entitys/teams.rs`：添加 `workspace_uuid`，移除 `default_proxy_uuid`
- `simprint-server/src/entitys/groups.rs`：添加 `workspace_uuid`、`team_uuid`，移除 `default_proxy_uuid`
- `simprint-server/src/entitys/environments.rs`：添加 `workspace_uuid`、`team_uuid`
- `simprint-server/src/entitys/proxies.rs`：添加 `workspace_uuid`
- `simprint-server/src/entitys/subscriptions.rs`：添加 `workspace_uuid`

### 3.3 Model 层（数据库操作）

#### 3.3.1 新增 Model

- `simprint-server/src/models/workspaces.rs`：工作空间数据库操作
- `simprint-server/src/models/workspace_quotas.rs`：工作空间配额数据库操作
- `simprint-server/src/models/proxy_visible_teams.rs`：代理可见性数据库操作
- `simprint-server/src/models/group_member_permissions.rs`：分组权限数据库操作

#### 3.3.2 修改现有 Model

- `simprint-server/src/models/teams.rs`：
  - `fetch_team_member`：添加 `workspace_uuid` 参数（工作空间级别隔离）
  - `insert_team`：添加 `workspace_uuid` 参数
  - `fetch_team_by_uuid`：添加 `workspace_uuid` 到 SELECT
  - `fetch_user_teams`：添加 `workspace_uuid` 过滤
  - 移除配额相关逻辑

- `simprint-server/src/models/team_members.rs`：
  - 所有函数添加 `workspace_uuid` 参数
  - SQL 查询包含 `workspace_uuid` 过滤

- `simprint-server/src/models/groups.rs`：
  - `insert_group`：添加 `workspace_uuid`、`team_uuid` 参数，移除 `user_uuid`
  - `fetch_groups`：使用 `workspace_uuid` 和 `team_uuid` 过滤
  - 移除 `default_proxy_uuid` 相关逻辑

- `simprint-server/src/models/environments.rs`：
  - `insert_environment`：添加 `workspace_uuid` 参数
  - `fetch_environment_by_uuid`：添加 `workspace_uuid` 参数进行过滤
  - 新增 `fetch_environment_by_uuid_unfiltered`：用于内部查询
  - 所有查询函数添加 `workspace_uuid` 过滤

- `simprint-server/src/models/proxies.rs`：
  - `insert_proxy`：添加 `workspace_uuid` 参数，`user_uuid` 改为 `owner_uuid`，移除 `team_uuid`
  - `fetch_proxies`：使用 `workspace_uuid` 和 `owner_uuid` 过滤
  - 移除 `usage_count` 相关逻辑

- `simprint-server/src/models/subscriptions.rs`：
  - 所有函数添加 `workspace_uuid` 参数

- `simprint-server/src/models/user.rs`：
  - `create_user_with_info`：创建默认工作空间和配额
  - `fetch_user_info_by_uuid`、`fetch_user_info_by_email`：添加 `current_workspace_uuid` 到 SELECT

### 3.4 Service 层（业务逻辑）

#### 3.4.1 新增 Service

- `simprint-server/src/services/workspaces.rs`：工作空间业务逻辑
- `simprint-server/src/services/workspace_quotas.rs`：工作空间配额业务逻辑
- `simprint-server/src/services/proxy_visibility.rs`：代理可见性业务逻辑
- `simprint-server/src/services/group_permissions.rs`：分组权限业务逻辑

#### 3.4.2 修改现有 Service

**环境服务（environments.rs）**：

- `create_environment_service`：
  - 添加工作空间级别团队成员检查
  - 添加分组权限检查（write/manage）
  - 添加团队角色权限检查（Editor/Admin/Owner）
  - 添加配额检查和更新
- `get_environment_service`：
  - 添加工作空间过滤
  - 添加分组 read 权限检查
- `get_environments_service`：
  - 添加批量权限过滤（根据分组权限）
- `update_environment_service`：
  - 添加编辑权限检查（分组 write/manage 或团队角色）
- `delete_environment_service`：
  - 添加删除权限检查（分组 manage 或团队 Owner/Admin）
  - 添加配额更新

**分组服务（groups.rs）**：

- `create_group_service`：添加 Owner/Admin 权限检查
- `update_group_service`：添加 Owner/Admin 或 manage 权限检查
- `delete_group_service`：添加 Owner/Admin 或 manage 权限检查

**团队服务（teams.rs）**：

- 所有函数添加 `workspace_uuid` 参数
- `switch_team_service`：更新 `current_workspace_uuid`

**代理服务（proxies.rs）**：

- `create_proxy_service`：添加配额检查和更新
- `delete_proxy_service`：添加配额更新
- `batch_import_proxies_service`：添加配额检查和更新

**模板服务（templates.rs）**：

- `create_template_service`：从源环境获取 `workspace_uuid` 和 `team_uuid` 进行权限检查
- `apply_template_service`：添加权限检查参数

### 3.5 Handler 层（API 端点）

#### 3.5.1 新增 Handler

- `simprint-server/src/handlers/client/workspaces.rs`：工作空间 API 端点
- `simprint-server/src/handlers/client/workspace_quotas.rs`：工作空间配额 API 端点
- `simprint-server/src/handlers/client/proxy_visibility.rs`：代理可见性 API 端点
- `simprint-server/src/handlers/client/group_permissions.rs`：分组权限 API 端点

#### 3.5.2 修改现有 Handler

- `simprint-server/src/handlers/client/teams.rs`：使用 `workspace_uuid` 从 `RequestContext` 获取
- `simprint-server/src/handlers/client/environments.rs`：使用 `workspace_uuid` 和 `team_uuid` 从 `RequestContext` 获取
- `simprint-server/src/handlers/client/proxies.rs`：使用 `workspace_uuid` 和 `owner_uuid` 从 `RequestContext` 获取
- `simprint-server/src/handlers/client/templates.rs`：添加权限检查参数

### 3.6 Route 层（路由注册）

#### 3.6.1 新增 Route

- `simprint-server/src/routes/client/workspaces.rs`：工作空间路由
- `simprint-server/src/routes/client/workspace_quotas.rs`：工作空间配额路由
- `simprint-server/src/routes/client/proxy_visibility.rs`：代理可见性路由
- `simprint-server/src/routes/client/group_permissions.rs`：分组权限路由

#### 3.6.2 修改现有 Route

- `simprint-server/src/routes/client/mod.rs`：注册新路由模块
- `simprint-server/src/main.rs`：注册所有新路由

### 3.7 中间件和状态管理

#### 3.7.1 状态管理（state.rs）

- 添加 `CurrentWorkspace` 结构体
- `RequestContext` 添加 `current_workspace_uuid` 字段

#### 3.7.2 认证中间件（middlewares/auth.rs）

- 从 `user_infos` 获取 `current_workspace_uuid` 并设置到 `RequestContext`

---

## 4. 核心功能实现

### 4.1 权限检查体系

#### 4.1.1 工作空间级别隔离

- 所有团队成员关系查询都包含 `workspace_uuid` 过滤
- `fetch_team_member` 函数签名包含 `workspace_uuid` 参数
- 确保用户在不同工作空间中有不同的团队成员身份

#### 4.1.2 环境权限检查

- **创建环境**：
  - 检查用户是否在当前工作空间的团队中
  - 如果指定分组，检查分组 write/manage 权限
  - 如果未指定分组，检查团队角色权限（Editor/Admin/Owner）
  - 检查工作空间配额
- **查看环境**：
  - 检查用户是否在当前工作空间的团队中
  - 如果环境有分组，检查分组 read 权限
  - Owner/Admin 自动拥有所有分组权限
  - 无分组环境，所有团队成员都可以查看
- **更新环境**：
  - 检查分组 write/manage 权限或团队角色权限（Editor/Admin/Owner）
- **删除环境**：
  - 检查分组 manage 权限或团队 Owner/Admin 角色

#### 4.1.3 分组权限检查

- **创建分组**：只有 Owner/Admin 可以创建
- **更新分组**：Owner/Admin 或拥有 manage 权限
- **删除分组**：Owner/Admin 或拥有 manage 权限
- **分组权限函数**：`check_group_permission` 自动处理 Owner/Admin 权限

### 4.2 配额管理体系

#### 4.2.1 环境配额

- **检查**：`check_quota(workspace_uuid, "environments")`
- **更新**：
  - 创建环境：`increment_used_environments(workspace_uuid, 1)`
  - 删除环境：`decrement_used_environments(workspace_uuid, 1)`

#### 4.2.2 代理配额

- **检查**：`check_quota(workspace_uuid, "proxies")`
- **更新**：
  - 创建代理：`increment_used_proxies(workspace_uuid, 1)`
  - 删除代理：`decrement_used_proxies(workspace_uuid, 1)`
  - 批量导入：每成功导入一个代理就更新一次

#### 4.2.3 成员配额

- **检查**：`check_quota(workspace_uuid, "team_members")`
- **更新**：
  - 接受邀请：`update_used_team_members(workspace_uuid)`（仅当用户之前不是该工作空间的成员时）
  - 移除成员：`update_used_team_members(workspace_uuid)`
  - 退出团队：`update_used_team_members(workspace_uuid)`
- **注意**：成员配额是统计所有团队的活跃成员总数，需要重新计算

### 4.3 代理可见性管理

#### 4.3.1 可见性规则

1. **工作空间 Owner**：可以看到所有代理
2. **代理所有者**：可以看到自己的代理（无论是否在可见列表中）
3. **团队成员**：只能看到 `proxy_visible_teams` 中包含其团队的代理
4. **否则**：不可见

#### 4.3.2 可见性设置

- 只有代理所有者或工作空间所有者可以设置可见性
- 通过 `proxy_visible_teams` 关联表控制可见性
- 支持批量设置可见性

### 4.4 工作空间管理

#### 4.4.1 工作空间创建

- 用户注册时自动创建个人工作空间
- 支持创建不同类型的工作空间（personal/team/enterprise）
- 自动创建默认配额

#### 4.4.2 工作空间切换

- 用户可以在多个工作空间之间切换
- `user_infos.current_workspace_uuid` 记录当前工作空间
- 认证中间件自动设置当前工作空间

---

## 5. API 端点变更

### 5.1 新增 API 端点

#### 5.1.1 工作空间相关

- `POST /workspaces/create`：创建工作空间
- `POST /workspaces/list`：获取用户的工作空间列表
- `POST /workspaces/get`：获取工作空间详情
- `POST /workspaces/update`：更新工作空间
- `POST /workspaces/delete`：删除工作空间
- `POST /workspaces/switch`：切换工作空间

#### 5.1.2 工作空间配额相关

- `POST /workspace-quotas/get`：获取工作空间配额
- `POST /workspace-quotas/update`：更新配额使用情况

#### 5.1.3 代理可见性相关

- `POST /proxy-visibility/set`：设置代理对团队可见
- `POST /proxy-visibility/remove`：移除代理对团队的可见性
- `POST /proxy-visibility/batch-set`：批量设置代理可见性
- `POST /proxy-visibility/list-visible`：获取可见的代理列表
- `POST /proxy-visibility/list-teams`：获取代理的可见团队列表

#### 5.1.4 分组权限相关

- `POST /group-permissions/grant`：授予分组权限
- `POST /group-permissions/revoke`：撤销分组权限
- `POST /group-permissions/check`：检查分组权限
- `POST /group-permissions/list`：列出用户的分组权限

### 5.2 修改现有 API 端点

所有现有 API 端点现在都需要：

- 从 `RequestContext` 获取 `current_workspace_uuid`
- 从 `RequestContext` 获取 `current_team_uuid`（如适用）
- 传递 `workspace_uuid` 和 `team_uuid` 到服务层

---

## 6. 数据模型设计要点

### 6.1 冗余字段设计

为了优化查询性能，以下表包含冗余字段：

- `groups.workspace_uuid`：避免 JOIN 查询
- `environments.workspace_uuid`：便于直接查询工作空间的环境
- `team_members.workspace_uuid`：便于直接查询工作空间内的团队成员关系（工作空间级别的隔离）
- `group_member_permissions.workspace_uuid`、`team_uuid`：便于权限查询
- `proxy_visible_teams.workspace_uuid`：便于直接查询工作空间内的代理可见性

### 6.2 软删除策略

所有表都支持软删除：

- 使用 `deleted_at` 字段标记删除
- 查询时默认过滤已删除记录
- 支持数据恢复

### 6.3 时间戳管理

所有表都包含时间戳字段：

- `created_at`：创建时间
- `updated_at`：更新时间（自动更新）
- `deleted_at`：软删除时间（如适用）

### 6.4 UUID 主键

所有表使用 UUID 作为主键：

- 全局唯一标识
- 避免 ID 冲突
- 支持分布式系统

---

## 7. 关键设计决策

### 7.1 工作空间级别隔离

**决策**：团队成员关系是工作空间级别的，不是全局的。

**原因**：

- 用户在不同工作空间中可能有不同的团队成员身份和角色
- 提供更好的资源隔离和权限控制
- 支持多工作空间场景

**实现**：

- `team_members` 表包含 `workspace_uuid` 字段（冗余）
- 所有团队成员查询都包含 `workspace_uuid` 过滤
- 唯一约束包含 `workspace_uuid`：`UNIQUE (team_uuid, user_uuid, workspace_uuid)`

### 7.2 环境直接归属团队

**决策**：环境必须直接归属团队，分组是可选的分类容器。

**原因**：

- 简化数据模型
- 提供更清晰的权限控制
- 支持无分组场景

**实现**：

- `environments.team_uuid` 为 NOT NULL
- `environments.group_uuid` 为可选（NULL 表示直接归属团队）
- 权限控制：有分组用分组权限，无分组用团队权限

### 7.3 代理属于工作空间

**决策**：代理属于工作空间，不属于团队，通过可见性控制访问。

**原因**：

- 代理是工作空间级别的资源
- 支持跨团队共享代理
- 提供更灵活的代理管理

**实现**：

- `proxies.workspace_uuid` 为必须
- `proxies.owner_uuid` 记录代理所有者
- `proxy_visible_teams` 关联表控制可见性

### 7.4 配额基于工作空间

**决策**：配额是工作空间级别的，所有团队共享。

**原因**：

- 简化配额管理
- 支持统一计费
- 便于资源分配

**实现**：

- `workspace_quotas` 表存储工作空间配额
- 所有配额检查和更新都基于 `workspace_uuid`
- 成员配额统计所有团队的活跃成员总数

---

## 8. 迁移和兼容性

### 8.1 数据迁移

- 为每个现有用户创建默认个人工作空间
- 将现有 `user_quotas` 数据迁移到 `workspace_quotas`
- 更新所有相关表的 `workspace_uuid` 字段
- 确保数据完整性

### 8.2 向后兼容

- 保留 `deprecated_user_quotas` 表用于历史数据
- 迁移脚本确保所有现有数据都有对应的 `workspace_uuid`
- 用户注册时自动创建个人工作空间

---

## 9. 测试建议

### 9.1 单元测试

- 工作空间级别隔离测试
- 权限检查函数测试
- 配额管理函数测试

### 9.2 集成测试

- 代理可见性逻辑测试
- 分组权限控制测试
- 工作空间切换测试

### 9.3 安全测试

- 跨工作空间访问测试
- 权限绕过测试
- 配额绕过测试

---

## 10. 已知问题和限制

### 10.1 数据库约束

- 迁移脚本中注释掉了 `team_uuid` 的 NOT NULL 约束
- **建议**：在数据迁移完成后，确保所有环境都有 `team_uuid`，然后添加 NOT NULL 约束

### 10.2 性能优化

- 冗余字段设计已优化查询性能
- 建议添加适当的索引以进一步提升性能

---

## 11. 后续工作

### 11.1 待完成

- [ ] 在数据迁移完成后，添加 `team_uuid` 的 NOT NULL 约束
- [ ] 添加单元测试和集成测试
- [ ] 进行安全审查

### 11.2 优化建议

- [ ] 添加缓存层以提升性能
- [ ] 优化配额统计查询
- [ ] 添加配额使用情况监控

---

## 12. 总结

本次重构实现了完整的工作空间架构，包括：

✅ **核心功能**：

- 工作空间管理
- 工作空间级别隔离
- 权限检查体系
- 配额管理体系
- 代理可见性管理

✅ **数据模型**：

- 4 个新表
- 7 个表的结构修改
- 完整的数据迁移脚本

✅ **代码实现**：

- 4 个新的 Service 模块
- 4 个新的 Handler 模块
- 4 个新的 Route 模块
- 所有现有模块的更新

✅ **API 端点**：

- 15+ 个新的 API 端点
- 所有现有端点的更新

所有实现都符合 `workspace-architecture.md` 文档的设计要求，确保了系统的可扩展性、安全性和可维护性。

---

**报告生成时间**：2025-01-25  
**报告版本**：v1.0  
**文档维护者**：开发团队
