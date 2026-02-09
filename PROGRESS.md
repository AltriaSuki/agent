# Process CLI Migration Progress

> **Last Updated**: 2026-02-09
> **架构愿景**: [DESIGN.md](DESIGN.md)
> **详细路线图**: [ROADMAP.md](ROADMAP.md)

## 项目概述

将 `legacy/v1/process-cli.sh` (1,522 行 Bash) 迁移为可扩展的 Rust CLI 工具。

## 当前状态: ~35% 完成

### ✅ 已完成

#### 基础架构
- [x] Cargo Workspace 设置 (6 crates)
- [x] 核心 crates: `process-core`, `process-ai`, `process-config`
- [x] CLI 框架 (clap) — 10 个子命令
- [x] 状态机 (`Phase` enum, `ProcessState`)
- [x] 配置系统 (默认值 → 全局 → 项目 → 环境变量)

#### 命令 (Phase 0-3)
- [x] `init` — 初始化项目
- [x] `seed-validate` — 验证 seed.yaml 6字段规范 ✨ NEW
- [x] `status` — 显示当前状态
- [x] `ai-config show/test` — AI 配置管理
- [x] `diverge` — 生成架构提案 (≥2个)
- [x] `diverge-validate` — 验证提案格式
- [x] `converge` — 收敛为单一方案 + 规则
- [x] `converge-validate` — 验证规则格式
- [x] `skeleton` — 生成项目骨架
- [x] `skeleton-validate` — 验证骨架输出

#### AI 系统
- [x] `AiProvider` trait + `CompletionRequest/Response`
- [x] `AiRegistry` (自动检测，按优先级排序)
- [x] `ClaudeProvider` (API) 实现

#### MS0: 基础修复 ✅ DONE (2026-02-09)
- [x] seed.yaml 模板对齐 PROCESS.md 6字段规范
- [x] 新增 `seed-validate` 命令（typed deserialization）
- [x] 统一 validate 行为：移除 phase 推进副作用
- [x] `ProcessState::load()` 无 .process/ 时报明确错误
- [x] `ai-config test` 复用 `utils::get_ai_provider()`
- [x] Claude 默认模型更新为 claude-sonnet-4-5
- [x] 清理 10 个未使用的 workspace 依赖

### ❌ 未实现

#### 从 Bash 迁移 (11 个命令)
- [ ] `branch new/start/review/abuse/gate/merge` — 分支工作流 (6个命令)
- [ ] `learn` / `friction` — 记录学习/摩擦点
- [ ] `stabilize` — 冻结不变量
- [ ] `postmortem` — AI 回顾
- [ ] `done` — 标记完成

#### 新增设计 (未实现)
- [ ] 适配已有项目 (`init --adopt`)
- [ ] Skeleton 应用 (`skeleton-apply`)
- [ ] 模板系统 (Tera)
- [ ] OpenAI / Ollama / Claude CLI / Manual Provider
- [ ] Generators (git-hooks, ci, makefile, ide)
- [ ] Checks (lint, test, sensitive, todo, audit)
- [ ] Reviews (general, security, performance, architecture)
- [ ] Command Trait 重构

### ⚠️ 已知 Bug
- (MS0 已全部修复，暂无已知 bug)

## 代码结构

```
agent/
├── crates/
│   ├── process-core/     # 状态机、Phase 定义 ✅
│   ├── process-ai/       # AI provider trait + registry ✅
│   ├── process-config/   # 配置系统 ✅
│   ├── process-checks/   # 自动化检查 ❌ placeholder
│   ├── process-generators/ # 生成器 ❌ placeholder
│   └── process-reviews/  # 审查模板 ❌ placeholder
├── src/
│   ├── commands/         # CLI 命令 (10/21 已实现)
│   ├── utils.rs          # 工具函数 (5 tests)
│   ├── cli.rs            # Clap 定义
│   └── main.rs
├── templates/            # ❌ 空目录
└── legacy/v1/            # 原 Bash 脚本 (参考)
```

## 运行命令

```bash
cargo build              # 构建 (零错误零警告)
cargo test               # 测试 (5/5 pass)
cargo run -- init        # 初始化
cargo run -- status      # 查看状态
cargo run -- diverge     # Phase 1
cargo run -- converge    # Phase 2
cargo run -- skeleton    # Phase 3
```

## 下次继续

**优先级从高到低** (详见 [ROADMAP.md](ROADMAP.md)):

1. **MS1: Bash 功能平替** — branch 命令组、learn/friction、stabilize/postmortem/done
2. **MS2: Adopt Pass 组** — 适配已有项目（scan-structure, scan-dependencies, infer-constraints, generate-seed, adopt-report）
3. **MS3: Prompt 系统** — 每个 AI provider 一套模板，Tera 模板引擎

## 配置 (config.yaml)

```yaml
ai:
  provider: claude
  claude:
    api_key: "YOUR_API_KEY"  # 或设置 ANTHROPIC_API_KEY 环境变量
    model: "claude-sonnet-4-5-20250929"
    base_url: "https://api.anthropic.com"
```
