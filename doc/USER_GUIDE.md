# Yuml DDT 使用指南

## 目录

- [快速开始](#快速开始)
- [YAML 配置详解](#yaml-配置详解)
- [测试步骤编写](#测试步骤编写)
- [环境管理](#环境管理)
- [变量系统](#变量系统)
- [认证配置](#认证配置)
- [验证规则](#验证规则)
- [高级功能](#高级功能)

---

## 快速开始

### 1. 安装应用

从 Release 页面下载对应平台的安装包：
- **macOS**: `.dmg` 文件
- **Windows**: `.msi` 或 `.exe` 文件
- **Linux**: `.deb` 或 `.AppImage` 文件

### 2. 创建第一个测试文件

创建一个名为 `test.yml` 的文件，内容如下：

```yaml
global:
  debug: true
  profile:
    active: "local"
    local:
      base_url: "https://jsonplaceholder.typicode.com"
      context: ""

steps:
  get_posts:
    name: "获取文章列表"
    method: "GET"
    path: "/posts"
    validate:
      - field: "[0].id"
        operator: "equals"
        expected: "1"
```

### 3. 打开并执行

1. 启动 Yuml DDT
2. 点击左侧 📂 图标，选择包含 `test.yml` 的目录
3. 在文件树中点击 `test.yml`
4. 点击 `get_posts:` 行左侧的 ▶ 按钮执行测试
5. 在右侧查看执行结果

---

## YAML 配置详解

### 配置文件结构

一个完整的 YAML 测试配置文件包含以下部分：

```yaml
global:           # 全局配置
  debug: bool     # 调试模式开关
  profile:        # 环境配置
    active: str   # 当前激活的环境
    <env_name>:   # 环境定义
      base_url: str
      context: str
      # ...

variables:        # 变量定义
  <key>: <value>

path_mapping:     # 路径映射（可选）
  <alias>: <path>

steps:            # 测试步骤
  <step_id>:
    name: str
    method: str
    path: str
    # ...

test_cases:       # 测试用例（可选）
  <case_id>:
    name: str
    steps: []
```

### global - 全局配置

全局配置包含调试开关和环境配置。

```yaml
global:
  debug: true                    # 开启调试模式，会在控制台输出详细日志
  profile:
    active: "uat"                # 当前激活的环境名称
    
    # 定义多个环境
    local:
      base_url: "http://localhost:8080"
      context: "/api"
      brand: "MK"
      
    dev:
      base_url: "https://dev-api.example.com"
      context: "/api/v1"
      brand: "MK"
      
    uat:
      base_url: "https://uat-api.example.com"
      context: "/api/v1"
      brand: "MK"
      auth:
        token_url: "https://uat-auth.example.com/token"
        username: "test_user"
        password: "test_pass"
        auth_position: "header"
        auth_key: "Authorization"
        auth_prefix: "Bearer "
```

#### 环境配置字段说明

| 字段 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `base_url` | 是 | API 基础地址 | `"https://api.example.com"` |
| `context` | 否 | 上下文路径，会拼接到所有请求前 | `"/api/v1"` |
| `brand` | 否 | 品牌标识，可在路径中引用 `{brand}` | `"MK"` |
| `auth` | 否 | 认证配置（详见[认证配置](#认证配置)） | - |
| `path_mapping` | 否 | 路径映射配置 | - |

### variables - 变量定义

定义可在测试中使用的变量：

```yaml
variables:
  userId: "user_123"
  productId: "prod_456"
  email: "test@example.com"
  baseAmount: 100
  isActive: true
```

使用变量：

```yaml
steps:
  get_user:
    path: "/users/{userId}"      # 使用 {变量名} 语法
    body:
      email: "{email}"
      amount: "{baseAmount}"
```

### path_mapping - 路径映射

简化重复的路径配置：

```yaml
path_mapping:
  user: "/v1/{brand}/user"
  order: "/v1/{brand}/order"
  product: "/v1/{brand}/product"

steps:
  get_user:
    path: "{user}/123"           # 实际路径：/v1/MK/user/123
    
  create_order:
    path: "{order}"              # 实际路径：/v1/MK/order
```

---

## 测试步骤编写

### 基本步骤结构

```yaml
steps:
  <step_id>:                     # 步骤唯一标识符
    name: "步骤名称"             # 可读的步骤名称
    method: "GET"                # HTTP 方法：GET/POST/PUT/DELETE
    path: "/api/resource"        # 请求路径
    params:                      # 查询参数（可选）
      key: "value"
    headers:                     # 请求头（可选）
      Content-Type: "application/json"
    body:                        # 请求体（可选，POST/PUT）
      field: "value"
    validate:                    # 验证规则（可选）
      - field: "code"
        operator: "equals"
        expected: "0"
    save:                        # 保存响应数据（可选）
      - from: "data.id"
        to: "newUserId"
```

### GET 请求示例

```yaml
steps:
  get_user_by_id:
    name: "根据ID获取用户"
    method: "GET"
    path: "/users/{userId}"
    params:
      include: "profile,settings"
    validate:
      - field: "data.id"
        operator: "equals"
        expected: "{userId}"
      - field: "data.email"
        operator: "not_empty"
```

### POST 请求示例

```yaml
steps:
  create_user:
    name: "创建新用户"
    method: "POST"
    path: "/users"
    headers:
      Content-Type: "application/json"
    body:
      name: "张三"
      email: "zhangsan@example.com"
      age: 25
      tags: ["vip", "active"]
    validate:
      - field: "code"
        operator: "equals"
        expected: "0"
      - field: "data.id"
        operator: "not_empty"
    save:
      - from: "data.id"
        to: "newUserId"
```

### PUT 请求示例

```yaml
steps:
  update_user:
    name: "更新用户信息"
    method: "PUT"
    path: "/users/{newUserId}"
    body:
      name: "李四"
      email: "lisi@example.com"
    validate:
      - field: "code"
        operator: "equals"
        expected: "0"
```

### DELETE 请求示例

```yaml
steps:
  delete_user:
    name: "删除用户"
    method: "DELETE"
    path: "/users/{newUserId}"
    validate:
      - field: "code"
        operator: "equals"
        expected: "0"
```

---

## 环境管理

### 多环境配置

在 `global.profile` 中定义多个环境：

```yaml
global:
  profile:
    active: "dev"              # 当前使用的环境
    
    local:                     # 本地环境
      base_url: "http://localhost:8080"
      context: ""
      
    dev:                       # 开发环境
      base_url: "https://dev.example.com"
      context: "/api"
      
    uat:                       # UAT 测试环境
      base_url: "https://uat.example.com"
      context: "/api"
      auth:
        token_url: "https://uat-auth.example.com/token"
        username: "uat_user"
        password: "uat_pass"
        
    prod:                      # 生产环境
      base_url: "https://api.example.com"
      context: "/api"
      auth:
        token_url: "https://auth.example.com/token"
        client_id: "prod_client"
        grant_type: "client_credentials"
```

### 环境切换

修改 `active` 字段即可切换环境：

```yaml
global:
  profile:
    active: "uat"              # 改为 uat 环境
```

### 环境特定配置

不同环境可以有不同的配置：

```yaml
global:
  profile:
    active: "dev"
    
    dev:
      base_url: "https://dev.example.com"
      brand: "DEV"
      path_mapping:
        user: "/v1/{brand}/user"
        
    prod:
      base_url: "https://api.example.com"
      brand: "MK"
      path_mapping:
        user: "/v2/{brand}/user"    # 生产环境使用 v2 API
```

---

## 变量系统

### 变量定义

在 `variables` 部分定义变量：

```yaml
variables:
  # 基本类型
  userId: "user_001"
  count: 10
  isActive: true
  
  # 对象（会被转为 JSON）
  userInfo:
    name: "张三"
    age: 25
```

### 变量引用

使用 `{变量名}` 语法引用变量：

```yaml
steps:
  get_user:
    path: "/users/{userId}"
    params:
      limit: "{count}"
    body:
      info: "{userInfo}"
```

### 内置变量

系统提供以下内置变量：

| 变量 | 说明 | 示例值 |
|------|------|--------|
| `{brand}` | 当前环境的品牌标识 | `"MK"` |
| `{timestamp}` | 当前时间戳（秒） | `"1707024000"` |
| `{uuid}` | 随机 UUID | `"550e8400-e29b-41d4-a716-446655440000"` |

使用示例：

```yaml
steps:
  create_order:
    body:
      orderId: "{uuid}"
      timestamp: "{timestamp}"
      brand: "{brand}"
```

### 动态变量（save）

从响应中提取数据并保存为变量：

```yaml
steps:
  create_user:
    method: "POST"
    path: "/users"
    body:
      name: "测试用户"
    save:
      - from: "data.id"          # 从响应的 data.id 字段提取
        to: "createdUserId"       # 保存为变量 createdUserId
      - from: "data.email"
        to: "userEmail"
        
  get_user_detail:
    method: "GET"
    path: "/users/{createdUserId}"    # 使用保存的变量
```

### 变量优先级

当变量名冲突时，优先级为：
1. 动态保存的变量（`save`）
2. 用户定义的变量（`variables`）
3. 内置变量（`brand`、`timestamp`、`uuid`）

---

## 认证配置

### Token 认证

在环境配置中添加 `auth` 部分：

```yaml
global:
  profile:
    active: "uat"
    uat:
      base_url: "https://api.example.com"
      auth:
        token_url: "https://auth.example.com/token"
        username: "test_user"
        password: "test_pass"
        auth_position: "header"           # token 位置：header/query
        auth_key: "Authorization"          # header 名称或参数名
        auth_prefix: "Bearer "             # token 前缀
```

### 认证配置字段

| 字段 | 说明 | 默认值 | 示例 |
|------|------|--------|------|
| `token_url` | Token 获取地址 | 必填 | `"https://auth.example.com/token"` |
| `request_format` | 登录请求编码 | `"form"` | `"form"` / `"json"` |
| `request_body` | 自定义登录请求体，整体替代默认 OAuth 字段 | 未设置 | `{email: "user@example.com", password: "secret"}` |
| `token_field` | 登录响应中 token 的字段路径，支持点分嵌套 | `"access_token"` | `"accessToken"` / `"data.accessToken"` |
| `username` | 用户名 | `""` | `"test_user"` |
| `password` | 密码 | `""` | `"test_pass"` |
| `client_id` | 客户端ID（OAuth） | `""` | `"client_123"` |
| `grant_type` | 授权类型 | `""` | `"client_credentials"` |
| `auth_position` | Token 发送位置 | `"header"` | `"header"` / `"query"` |
| `auth_key` | 键名 | `"Authorization"` | `"Authorization"` |
| `auth_prefix` | Token 前缀 | `"Bearer"` | `"Bearer "` |
| `token_cache_key` | 缓存键名 | `""` | `"my_token"` |

### JSON 登录与自定义 Token 字段

例如 Resume Builder 使用 JSON `email/password` 登录，响应字段为 `accessToken`：

```yaml
global:
  profile:
    active: local
    local:
      base_url: "http://127.0.0.1:8080"
      context: "/api"
      auth:
        token_url: "http://127.0.0.1:8080/api/auth/login"
        request_format: "json"
        request_body:
          email: "your-email@example.com"
          password: "your-password"
        token_field: "accessToken"
        auth_position: "header"
        auth_key: "Authorization"
        auth_prefix: "Bearer"
```

点击业务 step 时自动登录并携带 Token，无需手工复制，也无需在 step 中重复设置 Authorization。
`token_field` 决定从响应哪里取值；`auth_key` 决定发送时的 header 或 query 名称，两者独立。
登录地址使用完整 URL；认证配置使用字面值，不展开 `{variables}`。
`request_body` 未设置时仍发送原有 `client_id/grant_type/username/password` 字段，兼容旧配置。
JSON 请求体支持嵌套对象；form 请求体仅支持字符串、数字、布尔值。
登录 HTTP 失败、响应不是 JSON、Token 字段缺失或不是非空字符串时，当前 step 报错并停止发送业务请求。

### Token 位置示例

#### 1. Header 认证（推荐）

```yaml
auth:
  token_url: "https://auth.example.com/token"
  username: "user"
  password: "pass"
  auth_position: "header"
  auth_key: "Authorization"
  auth_prefix: "Bearer "
```

实际请求：
```
GET /api/users HTTP/1.1
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### 2. Query 参数认证

```yaml
auth:
  token_url: "https://auth.example.com/token"
  username: "user"
  password: "pass"
  auth_position: "query"
  auth_key: "access_token"
  auth_prefix: ""
```

实际请求：
```
GET /api/users?access_token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token 缓存

Token 会自动缓存，避免重复获取。缓存策略：
- 缓存在内存中
- 当前每次点击执行 step 都会新建引擎，因此会重新登录；缓存仅在同一引擎实例内复用，有效期一小时
- 可通过 `token_cache_key` 自定义缓存键名

---

## 验证规则

### 验证规则结构

```yaml
validate:
  - field: "响应字段路径"
    operator: "操作符"
    expected: "期望值"
```

### 字段路径语法

使用点号（`.`）和方括号（`[]`）访问嵌套字段：

```json
{
  "code": "0",
  "data": {
    "user": {
      "id": "123",
      "name": "张三"
    },
    "items": [
      {"id": 1, "name": "商品A"},
      {"id": 2, "name": "商品B"}
    ]
  }
}
```

字段路径示例：
- `"code"` → `"0"`
- `"data.user.id"` → `"123"`
- `"data.user.name"` → `"张三"`
- `"data.items[0].id"` → `1`
- `"data.items[1].name"` → `"商品B"`

### 支持的操作符

| 操作符 | 说明 | 示例 |
|--------|------|------|
| `equals` | 完全相等 | `expected: "0"` |
| `not_equals` | 不相等 | `expected: "error"` |
| `contains` | 包含子串 | `expected: "success"` |
| `not_contains` | 不包含子串 | `expected: "fail"` |
| `not_empty` | 非空（不需要 expected） | - |
| `is_empty` | 为空（不需要 expected） | - |
| `greater_than` | 大于 | `expected: "10"` |
| `less_than` | 小于 | `expected: "100"` |
| `regex` | 正则表达式匹配 | `expected: "^[0-9]+$"` |

### 验证示例

#### 基本验证

```yaml
steps:
  get_user:
    method: "GET"
    path: "/users/123"
    validate:
      # 验证响应码
      - field: "code"
        operator: "equals"
        expected: "0"
        
      # 验证非空
      - field: "data.name"
        operator: "not_empty"
        
      # 验证包含
      - field: "data.email"
        operator: "contains"
        expected: "@example.com"
```

#### 数值验证

```yaml
validate:
  # 大于
  - field: "data.age"
    operator: "greater_than"
    expected: "18"
    
  # 小于
  - field: "data.price"
    operator: "less_than"
    expected: "1000"
```

#### 数组验证

```yaml
validate:
  # 验证数组第一个元素
  - field: "data.items[0].id"
    operator: "equals"
    expected: "1"
    
  # 验证数组长度大于0
  - field: "data.items"
    operator: "not_empty"
```

#### 正则表达式验证

```yaml
validate:
  # 验证手机号格式
  - field: "data.phone"
    operator: "regex"
    expected: "^1[3-9]\\d{9}$"
    
  # 验证邮箱格式
  - field: "data.email"
    operator: "regex"
    expected: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
```

---

## 高级功能

### 测试用例（Test Cases）

将多个步骤组合成测试用例：

```yaml
steps:
  login:
    name: "用户登录"
    method: "POST"
    path: "/auth/login"
    body:
      username: "test"
      password: "123456"
    save:
      - from: "data.token"
        to: "authToken"
        
  create_order:
    name: "创建订单"
    method: "POST"
    path: "/orders"
    headers:
      Authorization: "Bearer {authToken}"
    body:
      amount: 100
    save:
      - from: "data.orderId"
        to: "orderId"
        
  query_order:
    name: "查询订单"
    method: "GET"
    path: "/orders/{orderId}"
    headers:
      Authorization: "Bearer {authToken}"

test_cases:
  complete_order_flow:
    name: "完整的订单流程"
    steps:
      - login
      - create_order
      - query_order
```

### 请求参数（Query Parameters）

```yaml
steps:
  search_users:
    method: "GET"
    path: "/users"
    params:
      keyword: "张三"
      page: "1"
      pageSize: "20"
      status: "active"
```

实际请求：
```
GET /users?keyword=张三&page=1&pageSize=20&status=active
```

### 自定义请求头

```yaml
steps:
  custom_headers:
    method: "POST"
    path: "/api/resource"
    headers:
      Content-Type: "application/json"
      X-Custom-Header: "custom-value"
      User-Agent: "Yuml-DDT/1.0"
    body:
      data: "test"
```

### 复杂请求体

#### 嵌套对象

```yaml
steps:
  create_complex:
    method: "POST"
    path: "/api/resource"
    body:
      user:
        name: "张三"
        age: 25
        address:
          city: "北京"
          district: "朝阳区"
      items:
        - id: 1
          name: "商品A"
          qty: 2
        - id: 2
          name: "商品B"
          qty: 1
```

#### 使用变量

```yaml
variables:
  userName: "李四"
  userAge: 30

steps:
  create_user:
    method: "POST"
    path: "/users"
    body:
      name: "{userName}"
      age: "{userAge}"
      metadata:
        createdAt: "{timestamp}"
        uuid: "{uuid}"
```

### 调试技巧

#### 1. 开启 Debug 模式

```yaml
global:
  debug: true                    # 开启调试输出
```

Debug 模式会在应用日志中输出：
- 请求的完整 URL
- 请求头
- 请求体
- 响应状态码
- 响应体

#### 2. 查看原始请求

右侧结果面板会显示实际发送的请求详情，包括：
- 实际的 URL（变量已替换）
- 实际的 Headers（包括认证 Token）
- 实际的 Body

#### 3. 验证失败定位

验证失败时，结果面板会显示：
- 哪个字段验证失败
- 期望值是什么
- 实际值是什么

---

## 常见问题

### 1. 文件无法打开

**问题**：选择目录后看不到 YAML 文件

**解决**：
- 确保文件扩展名是 `.yml` 或 `.yaml`
- 检查文件是否在所选目录或其子目录中
- 尝试刷新文件树

### 2. 步骤执行失败

**问题**：点击运行按钮后没有反应或报错

**解决**：
- 检查 YAML 语法是否正确（缩进、拼写）
- 开启 `debug: true` 查看详细日志
- 检查网络连接和 API 地址是否正确

### 3. Token 获取失败

**问题**：认证配置后仍然报 401 错误

**解决**：
- 检查 `token_url` 是否正确
- 检查 `username` 和 `password` 是否正确
- 检查 Token 返回的字段名（可能不是 `access_token`）
- 查看 Token API 的实际响应格式

### 4. 变量未替换

**问题**：请求中的 `{变量名}` 没有被替换

**解决**：
- 检查变量名拼写是否正确
- 确保变量已在 `variables` 中定义
- 如果使用 `save` 保存的变量，确保前序步骤已执行

### 5. 验证总是失败

**问题**：明明响应正确，但验证仍然失败

**解决**：
- 检查字段路径是否正确（大小写敏感）
- 检查期望值的类型（字符串 vs 数字）
- 使用 Debug 模式查看实际响应结构
- 尝试使用 `contains` 而不是 `equals`

---

## 最佳实践

### 1. 组织文件结构

```
tests/
├── common/
│   └── variables.yml          # 公共变量
├── user/
│   ├── user-create.yml        # 用户创建测试
│   ├── user-query.yml         # 用户查询测试
│   └── user-update.yml        # 用户更新测试
└── order/
    ├── order-create.yml       # 订单创建测试
    └── order-query.yml        # 订单查询测试
```

### 2. 使用有意义的命名

```yaml
# 好的命名
steps:
  create_user_with_profile:
    name: "创建用户并设置个人资料"
    
  query_active_orders:
    name: "查询所有活跃订单"

# 避免的命名
steps:
  step1:
    name: "test"
  step2:
    name: "test2"
```

### 3. 复用配置

使用路径映射减少重复：

```yaml
path_mapping:
  api_v1: "/api/v1/{brand}"
  user_api: "{api_v1}/user"
  order_api: "{api_v1}/order"

steps:
  get_user:
    path: "{user_api}/123"      # /api/v1/MK/user/123
  
  get_order:
    path: "{order_api}/456"     # /api/v1/MK/order/456
```

### 4. 分环境管理敏感信息

```yaml
global:
  profile:
    active: "dev"
    
    dev:
      auth:
        username: "dev_user"
        password: "dev_pass"
        
    prod:
      auth:
        username: "prod_user"
        password: "prod_pass"    # 生产环境使用不同的凭证
```

### 5. 添加详细注释

```yaml
steps:
  complex_request:
    name: "复杂的业务请求"
    method: "POST"
    path: "/api/business"
    body:
      # 用户基本信息
      userId: "{userId}"
      
      # 订单信息
      order:
        amount: 100           # 订单金额（分）
        currency: "CNY"       # 货币类型
        
    validate:
      # 验证业务码
      - field: "code"
        operator: "equals"
        expected: "0"         # 0 表示成功
```

---

## 更新日志

查看完整的[更新日志](../CHANGELOG.md)。

---

## 技术支持

- 提交 Issue: [GitHub Issues](https://github.com/yourname/yuml-ddt/issues)
- 查看文档: [README.md](../README.md)

---

<p align="center">Made with ❤️ by Yuml DDT Team</p>
