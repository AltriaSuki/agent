# Process CLI Migration Progress

> **Last Updated**: 2026-02-03 18:30 (公司)

## 项目概述

将 `legacy/v1/process-cli.sh` Bash 脚本迁移为 Rust CLI 工具。

## 当前状态

### ✅ 已完成

#### Phase 1: 基础架构
- [x] Cargo Workspace 设置
- [x] 核心 crates: `process-core`, `process-ai`, `process-config`
- [x] CLI 框架 (clap)
- [x] 状态机 (`Phase` enum)
- [x] 配置系统 (多层配置)

#### Phase 2: 核心命令
- [x] `init` - 初始化项目
- [x] `status` - 显示当前状态
- [x] `ai-config` - AI 配置管理
- [x] `diverge` - 生成架构提案 (≥2个)
- [x] `diverge-validate` - 验证提案格式
- [x] `converge` - 收敛为单一方案 + 规则
- [x] `converge-validate` - 验证规则格式

#### AI 系统
- [x] `AiProvider` trait
- [x] `AiRegistry` (自动检测)
- [x] `ClaudeProvider` 实现

### ⏳ 待完成

#### Phase 2 剩余
- [ ] `skeleton` - 生成项目骨架
- [ ] `skeleton-validate` - 验证骨架输出

#### Phase 3: 扩展
- [ ] `OpenAiProvider`
- [ ] `OllamaProvider`
- [ ] 更多验证规则

## 代码结构

```
agent/
├── crates/
│   ├── process-core/     # 状态、Phase 定义
│   ├── process-ai/       # AI provider trait + registry
│   └── process-config/   # 配置系统
├── src/
│   ├── commands/         # CLI 命令实现
│   │   ├── init.rs
│   │   ├── status.rs
│   │   ├── diverge.rs
│   │   ├── diverge_validate.rs
│   │   ├── converge.rs
│   │   └── converge_validate.rs
│   ├── utils.rs          # 工具函数
│   ├── cli.rs            # Clap 定义
│   └── main.rs
└── legacy/v1/            # 原 Bash 脚本 (参考)
```

## 关键文件

| 文件 | 用途 |
|------|------|
| `.process/seed.yaml` | 项目种子定义 |
| `.process/diverge_summary.yaml` | 架构提案 |
| `.process/rules.yaml` | 收敛后的规则 |
| `.process/config.yaml` | 本地配置 (gitignored) |

## 下次继续

1. 实现 `skeleton` 命令：
   - 读取 `rules.yaml`
   - 调用 AI 生成目录结构
   - 输出到 `.process/skeleton.yaml`

2. 实现 `skeleton-validate` 命令

## 运行命令

```bash
# 构建
cargo build

# 测试
cargo test

# 运行
cargo run -- init
cargo run -- diverge
cargo run -- diverge-validate
cargo run -- converge
cargo run -- converge-validate
cargo run -- status
```

## 配置 (config.yaml)

```yaml
ai:
  provider: claude
  claude:
    api_key: "YOUR_API_KEY"  # 或设置 ANTHROPIC_API_KEY 环境变量
    model: "claude-sonnet-4-5-20250929"
    base_url: "https://api.anthropic.com"
```
