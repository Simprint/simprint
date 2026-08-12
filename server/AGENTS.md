# Simprint Server 开发约束与规范

本文档定义了 Simprint Server 项目的开发约束、代码风格和最佳实践。

## 1. 数据库迁移文件命名规范

### 1.1 迁移文件创建命令

使用 `sqlx migrate add` 命令创建数据库迁移文件，命名格式如下：

```bash
# 创建表
sqlx migrate add create_{table_name}

# 删除表
sqlx migrate add delete_{table_name}

# 插入数据
sqlx migrate add insert_{table_name}
```

### 1.2 命名示例

- 创建 users 表：`sqlx migrate add create_users`
- 删除 users 表：`sqlx migrate add delete_users`
- 插入初始数据：`sqlx migrate add insert_initial_data`

### 1.3 迁移文件位置

所有迁移文件应位于 `migrations/` 目录下，文件名格式为：

```
{timestamp}_{migration_name}.sql
```

## 2. 代码风格与格式化

### 2.1 Rustfmt 配置

项目使用 `rustfmt` 进行代码格式化，配置文件为 `rustfmt.toml`。

**格式化命令**：

```bash
cargo fmt
```

**检查格式**：

```bash
cargo fmt -- --check
```

### 2.2 Clippy Linting

项目使用 `clippy` 进行代码静态分析和 linting，配置文件为 `clippy.toml`。

**运行 Clippy**：

```bash
cargo clippy
```

**自动修复**：

```bash
cargo clippy --fix
```

### 2.3 代码风格约定

- **函数和变量**：使用 `snake_case`
- **类型和结构体**：使用 `PascalCase`
- **常量**：使用 `SCREAMING_SNAKE_CASE`
- **模块**：使用 `snake_case`

## 3. 编译配置

### 3.1 编译选项

项目在 `Cargo.toml` 中配置了以下编译选项：

- **Edition**: Rust 2021
- **Max Width**: 100 字符
- **Tab Spaces**: 4 个空格
- **Newline Style**: Unix (LF)

### 3.2 编译命令

```bash
# 检查代码（不编译）
cargo check

# 编译项目
cargo build

# 编译发布版本
cargo build --release

# 运行项目
cargo run
```

## 4. 错误处理规范

- 优先使用 `Result<T, E>` 进行错误处理
- 使用 `thiserror` 或 `anyhow` 库进行错误处理（如需要）
- 避免使用 `unwrap()` 和 `expect()`，除非在测试代码中或确定不会失败的情况

## 5. 文档规范

- 所有公共 API 应使用文档注释（`///`）
- 复杂函数应添加文档说明参数和返回值
- 使用 `cargo doc` 生成项目文档

## 6. 测试规范

- 单元测试放在与被测试代码相同的文件中
- 集成测试放在 `tests/` 目录下
- 使用 `cargo test` 运行所有测试

## 7. 提交前检查清单

在提交代码前，确保：

- [ ] 代码已通过 `cargo fmt` 格式化
- [ ] 代码已通过 `cargo clippy` 检查，无警告
- [ ] 所有测试通过（`cargo test`）
- [ ] 迁移文件命名符合规范
- [ ] 公共 API 都有适当的文档注释

## 8. 其他说明

- 所有数据库对应类型的映射需要放到dto目录中
- 所有数据库操作的功能需要放到models目录中
- 具体功能实现和调用models层或者其他业务代码都应该放到services目录中
- 在handlers中接收的json参数定义应该放到entitys中，参数通常可以直接传递到services，然后到models层，下面是参数的示例:

  ```
  // entitys/{file_name}.rs
  #[derive(Deserialize, Serialize, Debug, Default)]
  pub struct Pagination {
      pub page: i64,
      pub page_size: i64,
  }

  // handlers/{file_name}.rs
  use crate::utils::Json;
  use crate::state::CurrentUser;

  // JSON参数: Json(payload): Json<Pagination>
  // user_uuid参数: Extension(current_user): Extension<CurrentUser>,
  pub async fn list(Json(payload): Json<Pagination>) /* TODO: */ {
      // TODO:
  }
  ```

- 通常都是post请求，且都是json参数
- models 的实现示例如下:

  ```
  use sqlx::{Error, Pool, Sqlite};

  pub async fn insert_user(
      pool: &Pool<Postgres>,
      payload: InsertUserRequestPayload
  ) -> Result<Uuid, Error> {
      let mut tx = pool.begin().await?;

      let user_uuid: Uuid = sqlx::query_scalar("insert into users default values returning uuid;")
          .fetch_one(&mut *tx)
          .await?;

      sqlx::query("insert into user_infos (phone, user_uuid) values($1, $2) returning id;")
          .bind(payload.phone)
          .bind(user_uuid)
          .execute(&mut *tx)
          .await?;

      sqlx::query("insert into user_passwords (password, user_uuid) values($1, $2) returning id;")
          .bind(payload.password)
          .bind(user_uuid)
          .execute(&mut *tx)
          .await?;

      sqlx::query("insert into user_avatars (resource_hash, user_uuid) values($1, $2) returning id;")
          .bind(payload.default_avatar)
          .bind(user_uuid)
          .execute(&mut *tx)
          .await?;

      tx.commit().await?;

      Ok(user_uuid)
  }

  pub async fn fetch_user_balance(
      pool: &Pool<Postgres>,
      user_uuid: Uuid,
  ) -> Result<Option<i64>, Error> {
      let rec = sqlx::query_scalar(
          r#"
          SELECT balance FROM user_balance
          WHERE user_uuid = $1 AND deleted_at IS NULL
          "#,
      )
      .bind(user_uuid)
      .fetch_optional(pool)
      .await?;

      Ok(rec)
  }
  ```

- sqlx的相关数据库操作通常不使用宏。
- 缓存相关示例:

  ```
  pub(crate) const REGISTER_CODE_CACHE_KEY: &str = "verification:code:register:send";
  pub(crate) const CODE_EXPIRATION: u64 = 60 * 3;

  /// 设置用户注册的验证码
  pub async fn set_register_code(
      svc_ctx: &SvcCtx,
      phone: &str,
      code: &str,
  ) -> Result<(), anyhow::Error> {
      let key = format!("{}:{}", REGISTER_CODE_CACHE_KEY, phone);
      let _: () = svc_ctx
          .redis
          .clone()
          .set_ex(key, code, CODE_EXPIRATION)
          .await?;
      Ok(())
  }

  ```
