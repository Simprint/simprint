# 数据库设计文档

本文档描述了 Simprint Server 项目的数据库表结构设计。

## 表结构定义规范

### users

用户基础信息表

存储用户的基础标识信息，采用 UUID 作为主键，支持全局唯一标识。

**字段说明**：

- `uuid` - UUID 类型，主键，全局唯一标识
- `id` - 字符串类型，用户 ID，唯一标识，可用于业务查询（如：USER001）
- `created_at` - 时间戳类型，创建时间，NOT NULL
- `updated_at` - 时间戳类型，更新时间，NOT NULL
- `deleted_at` - 时间戳类型，删除时间（软删除），可为 NULL

**索引**：

- 主键索引：`uuid`
- 唯一索引：`id`
- 索引：`deleted_at`（用于软删除查询优化）

**约束**：

- `id` 必须唯一且不为空
- `created_at` 和 `updated_at` 由数据库自动管理

### user_infos

用户详细信息表

存储用户的详细业务信息，包括登录凭证、联系方式、个人信息等。通过 `user_uuid` 与 `users` 表关联。

**字段说明**：

- `id` - 自增整数类型，主键
- `user_uuid` - UUID 类型，外键，关联 `users.uuid`，NOT NULL
- `nickname` - 字符串类型，昵称，可为 NULL
- `email` - 字符串类型，邮箱地址，NOT NULL，唯一索引，用于登录
- `phone` - 字符串类型，手机号，可为 NULL（可选字段）
- `password` - 字符串类型，密码（Argon2 加密存储），NOT NULL
- `avatar_hash` - 字符串类型，头像文件 hash（MinIO 中存储的文件名，无后缀），可为 NULL
- `status` - 字符串类型，用户状态（active/inactive/banned），默认 'active'，NOT NULL
- `created_at` - 时间戳类型，创建时间，NOT NULL
- `updated_at` - 时间戳类型，更新时间，NOT NULL
- `deleted_at` - 时间戳类型，删除时间（软删除），可为 NULL

**索引**：

- 主键索引：`id`
- 唯一索引：`user_uuid`（一对一关系）
- 唯一索引：`email`（邮箱唯一，NOT NULL，用于登录）
- 索引：`deleted_at`（用于软删除查询优化）
- 索引：`status`（用于状态查询）

**约束**：

- `user_uuid` 必须唯一，确保与 `users` 表一对一关系
- `email` 必须唯一且不为空（用于登录）
- `password` 必须不为空
- `status` 默认值为 'active'

**关于头像字段的说明**：

- `avatar_hash` 字段用于存储 MinIO 中头像文件的 hash 值（文件名，无后缀）
- 目前阶段不实现头像功能，该字段可为 NULL
- 未来实现头像功能时，头像文件将存储在 MinIO 的 `avatar_bucket` 中，文件名为 hash 值
- 获取头像 URL 时，通过配置的 `minio.resource_url` 和 `avatar_bucket` 拼接完整 URL

**表关系**：

- `user_infos.user_uuid` → `users.uuid`（一对一关系，级联删除）

## 设计说明

### 软删除策略

- 所有表都支持软删除，通过 `deleted_at` 字段标记
- 查询时默认过滤 `deleted_at IS NULL` 的记录
- 物理删除仅在必要时进行（如数据清理）

### 用户标识

- `users.uuid`：系统内部使用的全局唯一标识（UUID）
- `users.id`：业务层使用的用户 ID（字符串，如 USER001）
- `user_infos.phone`：用户登录使用的手机号

### 会话管理

- Refresh Token 存储在 Redis 中，不在数据库表中
- 会话相关信息（设备信息、登录时间等）可在 Redis 中管理
- 如需持久化会话记录，可考虑添加 `user_sessions` 表

### 验证码管理

- 验证码存储在 Redis 中，设置过期时间（如 5 分钟）
- 格式：`verification_code:{phone}:{type}`（type: register/reset_password）
- 不在数据库表中存储验证码信息

### 扩展性考虑

- 用户权限：如需权限系统，可添加 `user_permissions` 表
- 用户角色：如需角色系统，可添加 `user_roles` 表和关联表
- 登录历史：如需记录登录历史，可添加 `user_login_history` 表
- 设备管理：如需管理用户设备，可添加 `user_devices` 表
