# YUML-DDT

<p align="center">
  <strong>YAML Data-Driven Testing</strong><br>
  基于 YAML 配置的数据驱动测试桌面工具
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue" alt="Tauri 2.0">
  <img src="https://img.shields.io/badge/Vue-3.0-green" alt="Vue 3.0">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-yellow" alt="MIT License">
</p>

## ✨ 功能特性

- 📂 **文件管理** - 打开、浏览、编辑 YAML 测试配置文件
- ✏️ **智能编辑** - 集成 Monaco Editor，语法高亮、代码折叠、自动补全
- ▶️ **一键执行** - 在编辑器行号旁点击运行按钮，即可执行测试
- 📊 **实时结果** - 查看请求详情、响应数据、执行耗时
- ✅ **自动验证** - 支持多种断言操作符，自动验证响应结果
- 🔄 **变量替换** - 支持 `{variable}` 语法的动态变量替换
- 🌍 **多环境支持** - Profile 切换，轻松管理多套环境配置
- 🔐 **认证管理** - 自动获取和缓存 Token，支持多种认证方式

## 🖥️ 界面预览

```
┌─────────────────────────────────────────────────────────────┐
│ [活动栏] │ [侧边栏]      │ [YAML 编辑器]    │ [执行结果]    │
│   📁     │ 资源管理器    │ ▶ get_user:      │ ✅ 200 OK    │
│   🧪     │ └ customer/   │     name: "..."  │ 耗时: 125ms  │
│   ⚙️     │   └ test.yml  │ ▶ create_order:  │ {...}        │
│          │               │     method: POST │              │
├─────────────────────────────────────────────────────────────┤
│ 状态栏: /path/to/test.yml                      YAML │ UTF-8 │
└─────────────────────────────────────────────────────────────┘
```

## � 文档

- **[完整使用指南](doc/USER_GUIDE.md)** - 详细的 YAML 配置教程、测试步骤编写、环境管理等
- **[快速开始](#-安装与运行)** - 快速上手指南
- **[YAML 配置格式](#-yaml-配置格式)** - 配置文件语法说明

## �🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| **桌面框架** | Tauri 2.0 |
| **前端** | Vue 3 + TypeScript |
| **编辑器** | Monaco Editor |
| **后端** | Rust |
| **HTTP 客户端** | reqwest |
| **配置解析** | serde_yaml |

## 📦 安装与运行

### 环境要求

- Node.js 18+
- Rust 1.70+
- npm / pnpm / yarn

### 开发模式

```bash
# 克隆项目
git clone https://github.com/yourname/yuml-ddt.git
cd yuml-ddt

# 安装前端依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 构建发布包

```bash
# 构建生产版本
npm run tauri build
```

构建完成后，可在 `src-tauri/target/release/bundle/` 找到安装包：
- macOS: `.dmg` / `.app`
- Windows: `.msi` / `.exe`
- Linux: `.deb` / `.AppImage`

## 📁 项目结构

```
yuml-ddt/
├── src/                        # 前端源码 (Vue 3)
│   ├── components/
│   │   ├── ActivityBar.vue     # 左侧活动栏
│   │   ├── FileTree.vue        # 文件树组件
│   │   ├── StepList.vue        # 测试步骤列表
│   │   ├── YamlEditor.vue      # Monaco YAML 编辑器
│   │   └── ResultPanel.vue     # 执行结果面板
│   ├── App.vue                 # 主应用组件
│   ├── main.ts                 # 应用入口
│   └── style.css               # 全局样式
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # 库入口
│   │   ├── main.rs             # 应用入口
│   │   ├── config.rs           # 配置数据结构
│   │   ├── engine.rs           # 测试引擎核心
│   │   ├── http_client.rs      # HTTP 客户端
│   │   ├── cache.rs            # Token 缓存
│   │   └── commands.rs         # Tauri 命令
│   ├── Cargo.toml              # Rust 依赖
│   └── tauri.conf.json         # Tauri 配置
│
├── package.json                # 前端依赖
├── vite.config.ts              # Vite 配置
└── tsconfig.json               # TypeScript 配置
```

## 📝 YAML 配置格式

### 完整示例

```yaml
# 全局配置
global:
  debug: false
  profile:
    active: "local"           # 当前激活的环境
    
    local:                    # 本地开发环境
      base_url: "http://localhost:8080"
      context: "/api"
      brand: "MK"
      auth:
        token_url: "http://localhost:8080/auth/token"
        username: "admin"
        password: "admin123"
        auth_position: "header"    # token 发送位置: header/query
        auth_key: "Authorization"
        auth_prefix: "Bearer"
        token_field: "access_token" # 登录响应中读取 token 的字段
    
    dev:                      # 开发环境
      base_url: "https://dev-api.example.com"
      context: "/api"
      brand: "MK"

# 变量定义
variables:
  userId: "test_user_001"
  orderId: "ORDER_12345"

# 路径映射 (简化 URL 配置)
path_mapping:
  user: "/v1/{brand}/user"
  order: "/v1/{brand}/order"

# 测试步骤
steps:
  get_user:
    name: "获取用户信息"
    method: "GET"
    path: "{user}/{userId}"   # 使用路径映射和变量
    validate:
      - field: "code"
        operator: "equals"
        expected: "0"
      - field: "data.name"
        operator: "not_empty"
    save:
      - from: "data.id"
        to: "savedUserId"
  
  create_order:
    name: "创建订单"
    method: "POST"
    path: "{order}"
    body:
      userId: "{userId}"
      amount: 100
    headers:
      Content-Type: "application/json"
    validate:
      - field: "code"
        operator: "equals"
        expected: "0"
```

### 支持的验证操作符

| 操作符 | 说明 | 示例 |
|--------|------|------|
| `equals` | 相等 | `expected: "0"` |
| `not_equals` | 不相等 | `expected: "error"` |
| `contains` | 包含 | `expected: "success"` |
| `not_contains` | 不包含 | `expected: "fail"` |
| `not_empty` | 非空 | - |
| `is_empty` | 为空 | - |
| `greater_than` | 大于 | `expected: "10"` |
| `less_than` | 小于 | `expected: "100"` |
| `regex` | 正则匹配 | `expected: "^[0-9]+$"` |

### 变量替换

支持在以下位置使用 `{variableName}` 语法：

- `path` - URL 路径
- `body` - 请求体
- `headers` - 请求头
- `validate.expected` - 期望值

**内置变量：**
- `{brand}` - 当前环境的品牌标识
- `{timestamp}` - 当前时间戳
- `{uuid}` - 随机 UUID

## 🔧 使用说明

### 1. 打开测试目录

点击左侧活动栏的 📁 图标，然后点击 📂 按钮选择包含 YAML 测试文件的目录。

### 2. 选择测试文件

在文件树中点击 `.yml` 或 `.yaml` 文件，内容会加载到编辑器中。

### 3. 执行测试步骤

有两种方式执行测试：

- **方式一**：在编辑器中，每个 step 名称行的左侧会显示绿色 ▶ 按钮，点击即可执行
- **方式二**：点击左侧 🧪 图标，在测试列表中点击对应步骤的运行按钮

### 4. 查看执行结果

右侧面板会显示：
- 请求 URL、方法、Headers
- 请求 Body
- 响应状态码
- 响应 Body（JSON 格式化）
- 验证结果（通过/失败）
- 执行耗时

### 5. 保存修改

编辑 YAML 后，按 `⌘+S` (Mac) 或 `Ctrl+S` (Windows/Linux) 保存。

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| `⌘/Ctrl + S` | 保存文件 |
| `⌘/Ctrl + F` | 搜索 |
| `⌘/Ctrl + Z` | 撤销 |
| `⌘/Ctrl + Shift + Z` | 重做 |

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 License

[MIT License](LICENSE)

---

<p align="center">Made with ❤️ using Tauri + Vue + Rust</p>
