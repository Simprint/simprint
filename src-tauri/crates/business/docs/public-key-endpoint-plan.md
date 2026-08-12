# 获取公钥 API 端点实现计划（已更新）

## 概述

本计划用于实现 `GET /api/v1/secret/public/key` API 端点，该端点用于获取服务器的 RSA 公钥。客户端可以使用此公钥加密敏感数据，然后发送到服务器。

## 当前状态分析

### 已有功能

1. ✅ **路由配置**：路由已在配置文件中加入白名单（`configs/config.dev.toml` 和 `configs/config.prod.toml`）
2. ✅ **RSA 密钥工具**：`src/utils/secret/rsa.rs` 中已有 `RsaSecret` 结构和 `get_public_key()` 方法
3. ✅ **全局 RSA 实例**：`get_rsa_secret_instance()` 函数可以获取全局 RSA 密钥实例
4. ✅ **客户端实现参考**：`src-tauri` 中已有客户端获取公钥的实现，可作为参考

### 缺失功能

1. ❌ **Handler 函数**：缺少处理获取公钥请求的 handler 函数
2. ❌ **路由注册**：缺少注册 `/secret/public/key` 路由的代码
3. ❌ **Service 函数**：可选，可以直接在 handler 中调用工具函数

## 重要发现：客户端期望的响应格式

通过分析 `src-tauri/src/infrastructure/storage/credential.rs` 中的客户端实现，发现了关键信息：

### 客户端代码分析

```rust
// 客户端发送 GET 请求
let response = client.get(url).send().await?;

// 客户端期望纯文本响应（不是 JSON）
let public_key = response.text().await?;

// 客户端验证响应格式
if !public_key.contains("-----BEGIN RSA PUBLIC KEY-----") {
    return Err("服务器响应错误".to_string());
}
```

### 关键结论

1. **响应格式**：客户端期望**纯文本响应**（`text/plain`），而不是 JSON 格式
2. **响应内容**：直接返回 PEM 格式的公钥字符串，例如：
   ```
   -----BEGIN RSA PUBLIC KEY-----
   MIIBCgKCAQEA...
   -----END RSA PUBLIC KEY-----
   ```
3. **路由路径**：`secret/public/key`（相对路径，完整路径为 `/api/v1/secret/public/key`）
4. **请求拦截器**：客户端的请求拦截器会跳过 `/secret/public/key` 路径的加密处理（见 `src-tauri/src/infrastructure/network/interceptors/request.rs`）

## 实现方案

### 方案选择

由于获取服务器公钥是一个简单的操作，且客户端期望纯文本响应，建议：

- **直接在 handler 中调用工具函数**，无需创建额外的 service 层
- **返回纯文本响应**，使用 Axum 的 `PlainText` 或直接使用 `String` 实现 `IntoResponse`

### 目录结构

参考现有的 `time` 模块结构：

```
src/
├── handlers/
│   ├── time.rs          (已有)
│   └── secret.rs        (新建)
├── routes/
│   ├── time.rs          (已有)
│   └── secret.rs        (新建)
├── handlers.rs
└── routes.rs
```

## 实现步骤

### 步骤 1：创建 Handler 函数

**文件**：`src/handlers/secret.rs`

**功能**：

- 从全局 RSA 实例获取公钥
- 返回纯文本响应（PEM 格式的公钥字符串）

**代码结构**：

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use crate::utils::get_rsa_secret_instance;

pub async fn get_public_key_handler() -> impl IntoResponse {
    let public_key = get_rsa_secret_instance().get_public_key();
    (StatusCode::OK, public_key).into_response()
}
```

**说明**：

- 使用 `impl IntoResponse` 作为返回类型，可以返回纯文本
- `(StatusCode::OK, public_key)` 会创建一个状态码为 200 的纯文本响应
- Axum 会自动设置 `Content-Type: text/plain; charset=utf-8`

### 步骤 2：创建路由注册模块

**文件**：`src/routes/secret.rs`

**功能**：

- 注册 `/public/key` 路由
- 使用 GET 方法
- 绑定到 `get_public_key_handler`

**代码结构**：

```rust
use axum::routing::get;
use crate::handlers::get_public_key_handler;
use crate::routes::route::{self, MetaRoute};

pub fn register_routes(meta_route: &mut MetaRoute) -> () {
    let mut secret_route = route::RouteGroup::new("/secret");

    secret_route.add_route_item(route::RouteItem::get("/public/key", get(get_public_key_handler)));

    meta_route.add_route_group(secret_route);
}
```

### 步骤 3：注册模块

**文件**：`src/handlers.rs`

```rust
mod time;
mod secret;  // 新增

pub use time::*;
pub use secret::*;  // 新增
```

**文件**：`src/routes.rs`

```rust
mod time;
mod secret;  // 新增

pub fn all_routes(svc_ctx: &SvcCtx) -> Router<SvcCtx> {
    // ...
    time::register_routes(&mut meta_route);
    secret::register_routes(&mut meta_route);  // 新增
    // ...
}
```

### 步骤 4：验证和测试

1. **编译检查**：`cargo check`
2. **代码格式化**：`cargo fmt`
3. **Lint 检查**：`cargo clippy`
4. **功能测试**：
   - 启动服务器
   - 使用 curl 或 Postman 测试端点：
     ```bash
     curl http://localhost:40041/api/v1/secret/public/key
     ```
   - 验证返回的是 PEM 格式的公钥字符串
   - 验证响应头 `Content-Type` 为 `text/plain`

## API 响应格式

### 成功响应（HTTP 200）

**响应头**：

```
Content-Type: text/plain; charset=utf-8
```

**响应体**（纯文本）：

```
-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAseI9vA7iTxOMb5Y2xCL7BOGr1by9qEH4EfP9Bj90gxDmY8yRsVK/
o2g+i95oQxzdvdvpPAocKlQv2FEbZaqFr2Q4vy1cLrM0B1NTZl1/hGcmPSofLT9g
mnjzP60ikY40Dxq+YXAxXZ4s2M+9thNnFr4OydacHEPEkTklcBQBglopSXc1yqHU
ARyCQ3/VxQrfh215vIPgMg2f6PH741zXFaIJjucXR8wJVySo7aZhlBTOVz5GzV0b
aWh31zA47ivXh84OXIEI+CKDUSnvsa8SCRMRs8LgaO1Xktv4yCfHHpo8Zoy+KdW0
LJxP11G+3f9RsYRpjdmsleyHuYYS07suqwIDAQAB
-----END RSA PUBLIC KEY-----
```

### 错误处理

如果 RSA 实例未初始化，`get_rsa_secret_instance()` 会直接退出程序（在启动时已初始化，正常情况下不会发生）。

## 注意事项

1. **响应格式的重要性**：
   - ⚠️ **必须返回纯文本**，不能使用 `Response<T>` 类型（它会转换为 JSON）
   - 客户端使用 `response.text().await` 期望纯文本响应
   - 客户端会验证响应包含 `-----BEGIN RSA PUBLIC KEY-----`

2. **安全性**：
   - 该端点返回的是服务器的**公钥**，是公开信息，可以安全地暴露
   - 路由已在白名单中，无需认证即可访问
   - 客户端的请求拦截器会跳过此路径的加密处理

3. **性能**：
   - 公钥是静态数据，从内存中读取，性能开销很小
   - 无需数据库查询

4. **代码风格**：
   - 遵循项目现有的代码风格（参考 `time.rs` 的实现）
   - 但注意响应格式不同（纯文本 vs JSON）
   - 添加适当的文档注释

5. **测试环境差异**：
   - `config.test.toml` 中使用的是 `/api/v2/secret/public/key`
   - 如果需要支持 v2 版本，可以在同一 handler 中注册多个路由，或创建单独的 handler

## 客户端兼容性

### 客户端实现位置

- **文件**：`src-tauri/src/infrastructure/storage/credential.rs`
- **函数**：`fetch_server_public_key()`
- **调用时机**：应用启动时（`init_server_public_key()`）

### 客户端验证逻辑

```rust
// 1. 发送 GET 请求
let response = client.get(url).send().await?;

// 2. 获取纯文本响应
let public_key = response.text().await?;

// 3. 验证公钥格式
if !public_key.contains("-----BEGIN RSA PUBLIC KEY-----") {
    return Err("服务器响应错误".to_string());
}

// 4. 存储公钥
*SERVER_PUBLIC_KEY.write().unwrap() = Some(public_key.clone());
```

## 后续优化建议（可选）

1. **HTTP 缓存**：公钥是静态的，可以考虑添加 HTTP 缓存头（如 `Cache-Control: public, max-age=3600`）
2. **版本控制**：如果需要支持 v2 API，可以创建 `routes/secret_v2.rs`
3. **文档**：添加 OpenAPI/Swagger 文档（如果项目使用）

## 相关文件清单

### 需要创建的文件

- `src/handlers/secret.rs`
- `src/routes/secret.rs`

### 需要修改的文件

- `src/handlers.rs` - 添加 `mod secret;` 和 `pub use secret::*;`
- `src/routes.rs` - 添加 `mod secret;` 和调用 `secret::register_routes()`

### 相关配置文件（无需修改，已配置）

- `configs/config.dev.toml` - 已添加路由白名单
- `configs/config.prod.toml` - 已添加路由白名单
- `configs/config.test.toml` - 已添加路由白名单（v2 版本）

### 客户端参考代码

- `src-tauri/src/infrastructure/storage/credential.rs` - 客户端获取公钥的实现
- `src-tauri/src/infrastructure/network/interceptors/request.rs` - 请求拦截器（跳过公钥端点加密）
