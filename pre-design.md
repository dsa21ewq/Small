# Project `small`：极简项目环境引导工具 — 预设计文档 v3

> **版本**: MVP v0.1
> **最后更新**: 2026-05-25
> **状态**: 草稿
> **预估代码量**: 5,000-8,000 行 Rust

---

## 项目愿景

**"git clone && small install && small run。结束。"**

开源社区有大量小型项目——缺乏完善的 Dockerfile，README 环境说明滞后，新手常因依赖问题放弃。`small` 不做新的包管理器，不做可复现构建，不替代 Nix/Docker。它只做一件事：**把已有生态（pip/npm/apt/brew）拼起来，让开源项目的用户能跑起来。**

---

## 不做的事（比做什么更重要）

| 不做 | 原因 |
|---|---|
| **不做 lockfile** | small 不是 package manager。它在调 pip/npm，锁版本是 pip/npm 的职责 |
| **不做 checkpoint/resume** | partial state / corrupted state / idempotency 每个都是独立难题。MVP 重试就是重头跑 |
| **不做 sandbox (Docker)** | 引入一个和 small 本身一样复杂的依赖 |
| **不做 LLM / AI** | 没有真实失败样本数据库，AI 诊断是空中楼阁 |
| **不做 Windows 完整支持** | 不是 +20%，是 +200%。PowerShell quoting / UTF-16 / path escaping / symlink / antivirus / WSL boundary / MSVC / execution policy |
| **不做 trait / abstraction 层** | 先用硬编码跑通 50 个项目，再找真正的共性抽象 |
| **不做 validate --ci** | 直接输出 warning 就够了 |
| **不做 trust system** | 0 用户时没人关心 |

---

## 目标用户 & 核心场景

| 角色 | 场景 |
|---|---|
| **开发者** | `small init` 生成 `small.yaml`，提交到 Git |
| **终端用户** | `git clone` → `small install` → `small run`，结束 |

### MVP 硬约束

- **平台**: macOS + Linux
- **语言**: Python (pip) + Node.js (npm)
- **Windows**: 不做。文档里写"推荐 WSL2"。

---

## 核心设计决策

| # | 决策 | 结论 |
|---|------|------|
| 1 | **定位** | project bootstrap orchestrator，不是 universal package manager |
| 2 | **平台** | macOS + Linux。Windows 不进入 MVP |
| 3 | **语言** | Python (pip) + Node.js (npm) |
| 4 | **运行时** | 复用 python-build-standalone + Node 官方 tarball。下载 → 缓存 → 使用。不自己维护构建 |
| 5 | **系统依赖** | 调 apt/brew 自动安装。sudo 前提示确认。包名映射外置为 `data/packages.toml` |
| 6 | **Schema** | 声明式 YAML。`project` + `runtimes` + `system` + `entrypoint`。无 platform_overrides（MVP 不需要）|
| 7 | **small init** | 扫描 requirements.txt / package.json 预填充 + 交互式补充 |
| 8 | **执行流程** | plan → 展示 → 一次性确认（非逐步骤）→ exec → test |
| 9 | **失败处理** | 展示原始错误 + 简单 cleanup（删 venv/node_modules）。不做诊断引擎 |
| 10 | **测试策略** | benchmark 仓库：20-50 个真实开源项目。`small install` 必须全部通过 |
| 11 | **分发** | GitHub Releases 单文件二进制 + 一行 curl 脚本 |
| 12 | **代码风格** | 先硬编码，跑通 50 个项目后再抽象。不写 trait，不写多层 engine |

---

## 系统架构

```
small (Rust 单文件二进制, ~6MB)

    small install
        │
        ├── 1. 解析 small.yaml
        ├── 2. 检测 OS (macOS / Linux)
        ├── 3. 检查 Python/Node → 缺失则下载到 ~/.small/runtimes/
        ├── 4. 检查系统包 (cmake等) → 缺失则调 apt/brew
        ├── 5. 生成执行计划 → 终端展示
        ├── 6. 用户确认 [Y/N]
        ├── 7. 按顺序执行
        │     ├── 下载运行时
        │     ├── 安装系统包
        │     ├── 创建 venv / npm install
        │     ├── 安装依赖
        │     └── 执行 test 命令
        └── 8. 输出结果

    small init
        │
        ├── 扫描 requirements.txt / package.json
        ├── 交互式问卷
        └── 生成 small.yaml

    small check   → 只检查不安装
    small clean   → 删 .small_venv / node_modules
    small run     → 按 entrypoint 启动
```

---

## small.yaml Schema (v0.1)

```yaml
# small.yaml — 声明你的项目需要什么
# 由 `small init` 生成，可手动编辑

project:
  name: "my-project"         # [必填]
  language: python           # [必填] python | node

runtimes:
  python: ">=3.9,<3.13"      # language=python 时必填
  # node: ">=18"             # language=node 时必填

system:                       # [可选] 系统级依赖
  - cmake
  - pkg-config

dependencies:                 # [可选] 包依赖
  python:
    - "numpy>=1.24"
    - "torch>=2.0"
  # node:
  #   - "express@^4.18"

# [可选] 指向已有文件
requirements_file: "requirements.txt"
# package_file: "package.json"

# [可选] 环境变量
env:
  MY_API_KEY: "请填入你的 API Key"

# [可选] 安装前后脚本
pre_install:
  - "chmod +x ./scripts/setup.sh && ./scripts/setup.sh"
post_install:
  - "python -m spacy download en_core_web_sm"

# [可选] 验证命令
test: "python -m pytest tests/ -v"

# [必填] 启动命令
entrypoint: "python main.py --config config.yaml"
```

### Schema 原则

- **声明式**：写"需要 Python >= 3.9"，不写"怎么装 Python"
- **最少必填**：`project.name` + `project.language` + `entrypoint`。其余全可选
- **无平台差异字段**：MVP 不做 `platform_overrides`。macOS/Linux 的行为差异由 small 内置逻辑处理

---

## CLI 命令体系

| 命令 | 说明 |
|---|---|
| `small init` | 扫描项目文件 + 交互式生成 small.yaml |
| `small install` | 一键安装：检测 → 下载运行时 → 装系统包 → 装依赖 → test |
| `small check` | 只检查环境兼容性，不执行任何安装 |
| `small clean` | 删除 .small_venv / node_modules |
| `small run` | 按 entrypoint 启动项目 |
| `small version` | 版本号 |

### `small init` 流程

```
$ small init

? 项目名称: my-project
? 语言 [python / node]: python
? Python 版本要求 [>=3.9]: >=3.9,<3.13

已扫描 requirements.txt → 发现 8 个依赖:
  numpy, torch, fastapi, uvicorn, pydantic, ...

? 系统依赖 (检测到 CMakeLists.txt):
  > cmake

? 启动命令: python main.py
? 验证命令 (回车跳过): python -m pytest tests/

✅ 已生成 small.yaml
```

### `small install` 输出

```
$ small install

🔍 环境检查
  OS: macOS 14.5 (arm64) ✓
  Python: 未找到 (将自动下载 3.11.9)
  cmake: 未安装 (将使用 Homebrew 安装)

📦 安装计划
  1. 下载 Python 3.11.9 → ~/.small/runtimes/python/3.11.9/
  2. brew install cmake (需要确认)
  3. 创建 .small_venv
  4. pip install 8 个包
  5. python -m pytest tests/

执行? [Y/n]: y

⏳ 下载 Python 3.11.9... ✓ (28 MB, 缓存)
⏳ brew install cmake... ✓
⏳ 创建虚拟环境... ✓
⏳ pip install... ✓
⏳ pytest... ✓ 12 passed

🎉 完成。运行 small run 启动。
```

---

## Runtime Manager

### 策略

复用已有生态，不自己维护构建：

- **Python**: 下载 [python-build-standalone](https://github.com/astral-sh/python-build-standalone)（astral-sh 维护，也是 uv 用的）
- **Node.js**: 下载 [Node.js 官方 tarball](https://nodejs.org/dist/)

### 缓存

```
~/.small/runtimes/
├── python/
│   └── 3.11.9/
│       ├── bin/python3
│       └── ...
├── node/
│   └── 20.12.0/
│       ├── bin/node
│       └── ...
└── versions.toml    # { "python": {"3.11.9": {"url": "...", "sha256": "..."}} }
```

### 行为

- 检查系统 PATH 中是否有满足版本约束的 Python/Node
- 有 → 直接使用系统版本
- 无 → 下载到 `~/.small/runtimes/`，安装期间加入 PATH
- `small clean` 不删除运行时缓存（跨项目共享）

### 版本数据

`versions.toml` 随 small 二进制一起分发（编译时 `include_bytes!` 嵌入）：

```toml
[python."3.11.9".macos_arm64]
url = "https://github.com/astral-sh/python-build-standalone/releases/download/20240224/cpython-3.11.9+20240224-aarch64-apple-darwin-install_only.tar.gz"
sha256 = "abc123..."

[python."3.11.9".linux_x64]
url = "https://..."
sha256 = "def456..."

[node."20.12.0".macos_arm64]
url = "https://nodejs.org/dist/v20.12.0/node-v20.12.0-darwin-arm64.tar.gz"
sha256 = "..."
```

---

## 系统包管理器

### 策略

简单直接：检测 PM → 调 PM。不做 trait 抽象，两个函数搞定。

```rust
fn detect_pm() -> Option<Pm> {
    if which("brew").is_ok()   { Pm::Brew }
    if which("apt-get").is_ok(){ Pm::Apt }
    // 后续加: dnf, pacman
    None
}

fn install_packages(pm: Pm, packages: &[String]) -> Result<()> {
    match pm {
        Pm::Brew => run("brew", ["install", packages]),
        Pm::Apt  => run("sudo", ["apt-get", "install", "-y", packages]),
    }
}
```

### 包名映射

`packages.toml` 编译时嵌入：

```toml
[cmake]
brew = "cmake"
apt = "cmake"

[pkg-config]
brew = "pkg-config"
apt = "pkg-config"

[python3-dev]
apt = "python3-dev"
# brew 不需要 (Xcode CLT 自带)
```

MVP 只维护 macOS (brew) + Ubuntu/Debian (apt) 的映射。Fedora/Arch 用户社区 PR 贡献。

---

## 执行引擎

### 流程

```
small install
  │
  ├── parse_yaml()        → SmallYaml
  ├── check_os()          → OsInfo
  ├── check_python()      → RuntimeStatus (found / need_download)
  ├── check_node()        → RuntimeStatus
  ├── check_system_pkgs() → Vec<SystemPkg> (found / need_install)
  ├── build_plan()        → Vec<Step>
  ├── print_plan()        → 终端展示
  ├── confirm()           → Y/N
  ├── execute(plan)       → 依次执行
  │     ├── download_runtime()
  │     ├── install_system_pkg()
  │     ├── create_venv()
  │     ├── pip_install()
  │     ├── npm_install()
  │     └── run_test()
  └── cleanup_on_failure() → rm -rf .small_venv / node_modules
```

### 不做 undo 栈

设计文档 v1/v2 里都有 undo 栈概念。MVP 不做。失败时：

1. 打印原始错误输出
2. `rm -rf .small_venv node_modules`（如果有）
3. 系统包 **不回退**（用户确认过的操作）

### 不做状态机

不需要 formal state machine。一个函数 `execute_plan()`，for 循环 + match 错误即可。

---

## Rust 工程结构

```
small/
├── Cargo.toml
├── data/
│   ├── versions.toml        # 运行时下载 URL + SHA256
│   └── packages.toml        # 系统包名映射
├── src/
│   ├── main.rs              # CLI 入口 + 子命令分发
│   ├── yaml.rs              # small.yaml 解析 (serde)
│   ├── init.rs              # small init: 扫描 + 交互式
│   ├── install.rs           # small install 主逻辑
│   ├── runtime.rs           # 运行时检测 + 下载
│   ├── syspkg.rs            # 系统包安装 (apt/brew)
│   ├── plan.rs              # 执行计划生成
│   ├── executor.rs          # 步骤执行
│   ├── check.rs             # small check
│   ├── clean.rs             # small clean
│   ├── run.rs               # small run
│   └── display.rs           # 终端输出 (颜色、进度)
├── tests/
│   ├── yaml_test.rs
│   ├── plan_test.rs
│   ├── runtime_test.rs
│   └── fixtures/
│       ├── python_basic.yaml
│       └── node_basic.yaml
├── benchmarks/               # benchmark 项目仓库（独立维护）
│   └── README.md             # 列出 20-50 个验证通过的开源项目
└── examples/
    ├── python-fastapi.small.yaml
    └── node-express.small.yaml
```

### 核心依赖

| Crate | 用途 |
|---|---|
| `clap` | CLI |
| `serde` + `serde_yaml` | YAML |
| `toml` | 数据文件解析 |
| `dialoguer` | 交互式输入 |
| `reqwest` + `tokio` | HTTP 下载 |
| `flate2` + `tar` | 解压运行时 |
| `sha2` | SHA256 校验 |
| `which` | 检测系统工具 |
| `duct` | 执行 shell 命令 |
| `indicatif` | 进度条 |
| `console` | 终端颜色 |

---

## Benchmark 策略（第一优先级）

### 比任何功能都重要

`small install` 必须在以下真实开源项目上通过：

```
benchmarks/python/
├── fastapi          # Web 框架
├── django           # Web 框架
├── flask            # 微框架
├── requests         # HTTP 库
├── numpy            # 科学计算
├── torch-minimal    # PyTorch 最小示例
├── scikit-learn     # 机器学习
├── pandas           # 数据分析
├── pillow           # 图像处理
├── celery           # 任务队列

benchmarks/node/
├── express          # Web 框架
├── nextjs-minimal   # Next.js 最小示例
├── react            # React
├── vue              # Vue
├── vite             # 构建工具
├── prisma           # ORM
├── tailwindcss      # CSS 框架
├── nest             # 后端框架

benchmarks/system-deps/
├── cmake-project    # 需要 cmake
├── openssl-project  # 需要 libssl-dev
├── gcc-project      # 需要 gcc
```

### 验证标准

每个 benchmark 项目：
1. `small install` → 成功
2. `small run` → 项目启动
3. `small test` → 测试通过
4. `small clean` → 环境恢复

CI 每次 PR 跑全量 benchmark。

### 失败日志收集

`small install` 失败时输出写到 `~/.small/logs/<project>-<timestamp>.log`。这是我们诊断引擎的数据基础。没有这些数据，AI/规则诊断都是空想。

---

## 路线图

### Phase 1: 跑通 50 个项目 (4-6 周, ~5000 行)

- [ ] small.yaml 解析
- [ ] small init（扫描 + 交互式）
- [ ] Runtime 检测 + 下载（python-build-standalone + Node tarball）
- [ ] 系统包安装（apt + brew）
- [ ] 执行计划生成 + 执行
- [ ] small install / check / clean / run
- [ ] CI 编译 macOS + Linux 二进制
- [ ] GitHub Releases 分发 + curl 安装脚本
- [ ] **Benchmark: 前 20 个 Python 项目通过**
- [ ] **Benchmark: 前 20 个 Node 项目通过**
- [ ] 收集失败日志

### Phase 2: 稳定 + 扩展 (6-8 周)

- [ ] 基于真实失败日志改进错误信息
- [ ] dnf (Fedora) + pacman (Arch) 支持
- [ ] 简单的错误模式匹配（基于收集的数据，不是猜的）
- [ ] Benchmark 扩展到 50 个项目
- [ ] Homebrew tap 分发
- [ ] npm 包分发

### Phase 3: 智能 (基于数据后, 8-12 周)

- [ ] Diagnostic engine（规则匹配，基于真实数据）
- [ ] 可选的 AI 诊断（有数据后再接 LLM）
- [ ] conditional 系统包映射（某些包只在 Linux 需要）
- [ ] Windows 实验性支持

### Phase 4: 生态 (后期)

- [ ] checkpoint/resume（当失败模式明确后）
- [ ] lockfile（当用户真正需要可复现时）
- [ ] Conda / Poetry / pnpm / yarn 支持
- [ ] VSCode 插件
- [ ] 社区 small.yaml 共享库

---

## 风险分析

| 风险 | 缓解 |
|---|---|
| **runtime 下载源不可用** | fallback 到系统 Python/Node 并警告 |
| **python-build-standalone 覆盖不全** | 先只支持 macOS arm64 + Linux x64 的主流版本 |
| **apt/brew 包名差异** | packages.toml 可手动更新；CI 中验证映射正确性 |
| **开发者不写 small.yaml** | small init 自动扫描降到最低门槛；target 开发者社区的 Python/Node 项目 |
| **Rust 编译体积** | LTO + strip，目标 5-8 MB |
| **"又一个工具"疲劳** | 定位清晰：不做新的包管理器，只做已有生态的 glue |
| **被 uv/devbox/devy 覆盖** | 差异化在"开源项目 onboarding"单一体验，而非功能矩阵 |

---

## 设计原则

1. **拼已有生态，不重新发明** — small 调 pip/npm/apt/brew，不替代它们
2. **先硬编码，再抽象** — 跑通 50 个项目后再找真正的模式
3. **benchmark 驱动开发** — 新功能由"哪个真实项目过不了"驱动，不由设计文档驱动
4. **失败日志是唯一数据资产** — 先收集数据，再做诊断/AI
5. **一条命令体验** — `git clone && small install && small run`，不允许中间步骤
