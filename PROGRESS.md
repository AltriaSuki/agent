# Process CLI Migration Progress

> **Last Updated**: 2026-02-10
> **架构愿景**: [DESIGN.md](DESIGN.md)
> **详细路线图**: [ROADMAP.md](ROADMAP.md)

## 项目概述

将 `legacy/v1/process-cli.sh` (1,522 行 Bash) 迁移为可扩展的 Rust CLI 工具。

## 当前状态: ~60% 完成

### ✅ 已完成

#### 基础架构
- [x] Cargo Workspace 设置 (6 crates)
- [x] 核心 crates: `process-core`, `process-ai`, `process-config`
- [x] CLI 框架 (clap) — 21 个子命令
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

#### MS1: Bash 功能平替 ✅ DONE (2026-02-10)
- [x] `learn` 命令 — 向 learnings.yaml 追加记录
- [x] `friction` 命令 — 向 friction.yaml 追加记录
- [x] `branch new` 命令 — 创建分支假设 YAML
- [x] `branch start` 命令 — 验证假设、创建 git 分支
- [x] `branch review` 命令 — 多角色 AI 审查 (4角色)
- [x] `branch abuse` 命令 — 恶意用户对抗测试
- [x] `branch gate` 命令 — 合并门控检查
- [x] `branch merge` 命令 — 标记分支为已合并
- [x] `stabilize` 命令 — 冻结不变量、检查未合并分支
- [x] `postmortem` 命令 — AI 生成回顾报告
- [x] `done` 命令 — 标记项目完成
- [x] 增强 `status` 命令 — 进度条 + 分支状态 + artifact 检查
- [x] 修复 process-ai clippy warning (sort_by_key)

#### MS1.5: 决策深度增强 ✅ DONE (2026-02-10)
- [x] 1.5a: 决策日志自动化 — `decision_log.rs` 模块，diverge/converge/skeleton/stabilize 集成
- [x] 1.5b: Challenge 命令 — `diverge-challenge` + `converge-challenge`
- [x] 1.5c: 冲突裁决增强 — `branch review` 自动检测角色冲突，交互式裁决
- [x] 1.5d: 决策质量回顾 — `postmortem` 交互式回顾每个历史决策

#### MS2: Adopt Pass 组 ✅ DONE (2026-02-10)
- [x] `adopt_utils.rs` — 共享工具 (ensure_process_dir, IGNORE_DIRS, detect_language, detect_frameworks, classify_file)
- [x] `adopt scan-structure` — 目录扫描，产出 skeleton.yaml (sync, no AI)
- [x] `adopt scan-dependencies` — 依赖分析，产出 seed.yaml (sync, no AI, 支持 Cargo.toml/package.json/requirements.txt/go.mod/pyproject.toml)
- [x] `adopt infer-conventions` — AI 推断编码规范，产出 rules.yaml
- [x] `adopt scan-git-history` — AI git 考古，产出 decisions_log.yaml
- [x] `adopt gap-analysis` — AI 差距分析，产出 gap-report.yaml
- [x] `adopt all` — 编排器，顺序运行所有 pass，设置状态为 Skeleton
- [x] 新增依赖: walkdir, toml, serde_json
- [x] CLI 嵌套子命令 (AdoptCommands enum)

### ❌ 未实现

#### 新增设计
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
│   ├── commands/         # CLI 命令 (27个, 含 adopt 子命令组)
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

1. **MS3: Prompt 系统** — 每个 AI provider 一套模板，Tera 模板引擎
2. **MS4: 测试体系** — 单元测试、集成测试、CI
3. **MS5: Provider 扩展** — OpenAI / Ollama / Claude CLI / Manual Provider

## 配置 (config.yaml)

```yaml
ai:
  provider: claude
  claude:
    api_key: "YOUR_API_KEY"  # 或设置 ANTHROPIC_API_KEY 环境变量
    model: "claude-sonnet-4-5-20250929"
    base_url: "https://api.anthropic.com"
```
