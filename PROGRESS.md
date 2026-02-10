# Process CLI Migration Progress

> **Last Updated**: 2026-02-10
> **架构愿景**: [DESIGN.md](DESIGN.md)
> **详细路线图**: [ROADMAP.md](ROADMAP.md)

## 项目概述

将 `legacy/v1/process-cli.sh` (1,522 行 Bash) 迁移为可扩展的 Rust CLI 工具。

## 当前状态: ~75% 完成

### ✅ 已完成

#### 基础架构
- [x] Cargo Workspace 设置 (6 crates)
- [x] 核心 crates: `process-core`, `process-ai`, `process-config`
- [x] CLI 框架 (clap) — 22 个子命令
- [x] 状态机 (`Phase` enum, `ProcessState`)
- [x] 配置系统 (默认值 → 全局 → 项目 → 环境变量)

#### 命令 (Phase 0-3)
- [x] `init` — 初始化项目
- [x] `seed-validate` — 验证 seed.yaml 6字段规范
- [x] `status` — 显示当前状态（进度条 + 分支状态 + artifact 检查）
- [x] `ai-config show/test/set-provider` — AI 配置管理 ✨ UPDATED
- [x] `diverge` — 生成架构提案 (≥2个)
- [x] `diverge-validate` — 验证提案格式
- [x] `converge` — 收敛为单一方案 + 规则
- [x] `converge-validate` — 验证规则格式
- [x] `skeleton` — 生成项目骨架
- [x] `skeleton-validate` — 验证骨架输出

#### MS0-MS2 ✅ DONE (详见 ROADMAP.md)

#### MS3: Prompt 系统 ✅ DONE (2026-02-10)
- [x] Tera 模板引擎 — 从 `format!()` 迁移到 `.md.tera` 文件
- [x] 4 级查找链 — 项目本地 provider → 项目本地 default → 内置 provider → 内置 default
- [x] `include_dir!` 编译时嵌入 — 12 个内置模板 (9 default + 3 claude)
- [x] 变量注入 — 所有命令使用 `tera::Context`
- [x] Claude 专用模板 — `diverge`/`converge`/`skeleton` 使用 XML 标签

#### MS4: 测试体系 ✅ DONE (2026-02-10)
- [x] `process-core` 单元测试 (11 tests) — Phase 排序、set_phase 只进不退、load/save 生命周期
- [x] `process-config` 单元测试 (3 tests) — 默认值正确性、无文件时使用默认
- [x] `process-ai` 单元测试 (9 tests) — MockProvider、registry auto-detect 优先级
- [x] `process-cli` 单元测试 (9 tests) — PromptEngine 渲染、utils strip_markdown
- [x] **共 32 个测试，全部通过** ✨ NEW

#### MS5: 更多 AI Provider ✅ DONE (2026-02-10)
- [x] OpenAI Provider — GPT-4o，`OPENAI_API_KEY`，priority 80
- [x] Ollama Provider — 本地模型 `llama3.1`，自动 ping 检测，priority 30
- [x] Claude CLI Provider — 调用 `claude` 命令行，priority 95 ✨ NEW
- [x] Manual Provider — 打印 prompt、等待粘贴，零依赖兜底，priority 1 ✨ NEW
- [x] `ai-config set-provider` 子命令 ✨ NEW
- [x] `ai-config show` 增强 — 显示所有已注册 provider 可用性 ✨ NEW

### AI 系统
- [x] `AiProvider` trait + `CompletionRequest/Response`
- [x] `AiRegistry` (auto-detect, 按优先级排序, `provider_exists()`)
- [x] 5 个 Provider: Claude API (90) → Claude CLI (95) → OpenAI (80) → Ollama (30) → Manual (1)

### ❌ 未实现

- [ ] Skeleton 应用 (`skeleton-apply`)
- [ ] Generators (git-hooks, ci, makefile, ide) — MS6
- [ ] Checks (lint, test, sensitive, todo, audit) — MS6
- [ ] Reviews (general, security, performance, architecture) — MS8
- [ ] Pass Engine 重构 — MS7
- [ ] CI 配置 (GitHub Actions) — MS4 附属
- [ ] `process pass run` / `process pass list` — MS7

### ⚠️ 已知 Bug
- (暂无已知 bug)

## 代码结构

```
agent/
├── crates/
│   ├── process-core/     # 状态机、Phase 定义 ✅ (11 tests)
│   ├── process-ai/       # AI provider trait + 5 providers ✅ (9 tests)
│   ├── process-config/   # 配置系统 ✅ (3 tests)
│   ├── process-checks/   # ❌ placeholder
│   ├── process-generators/ # ❌ placeholder
│   └── process-reviews/  # ❌ placeholder
├── src/
│   ├── commands/         # CLI 命令 (28个, 含 adopt 子命令组)
│   ├── prompts.rs        # PromptEngine (4级查找链)
│   ├── utils.rs          # 工具函数 + AI registry 构建
│   ├── cli.rs            # Clap 定义
│   └── main.rs
├── templates/prompts/    # Tera 模板
│   ├── _default/         # 9 个默认模板
│   └── claude/           # 3 个 Claude 专用模板
└── legacy/v1/            # 原 Bash 脚本 (参考)
```

## 运行命令

```bash
cargo build              # 构建 (零错误零警告)
cargo test --workspace   # 测试 (32/32 pass)
cargo run -- init        # 初始化
cargo run -- ai-config show         # 查看 AI 配置
cargo run -- ai-config set-provider openai  # 切换 provider
cargo run -- diverge     # Phase 1
```

## 下次继续

**优先级从高到低** (详见 [ROADMAP.md](ROADMAP.md)):

1. **MS6: Generators & Checks** — git-hooks, CI, lint, test 自动化
2. **MS7: Pass Engine** — 架构转型，从散装命令到 Pass DAG
3. **MS8: 打磨 & 发布** — Reviews, 帮助系统, Shell 补全, 发布

## 配置 (config.yaml)

```yaml
ai:
  provider: auto  # auto | claude | openai | ollama | claude-cli | manual
  claude:
    api_key: "YOUR_KEY"  # 或 ANTHROPIC_API_KEY 环境变量
    model: "claude-sonnet-4-5-20250929"
  openai:
    api_key: "YOUR_KEY"  # 或 OPENAI_API_KEY 环境变量
    model: "gpt-4o"
  ollama:
    base_url: "http://localhost:11434"
    model: "llama3.1"
```
