# Process CLI

> **在 AI 能瞬间生成一切的时代，人很容易变成橡皮图章。**
> **Process CLI 不是帮你更快写代码的工具，而是帮你成为更好的决策者。**

Process CLI 是一个通用决策流水线引擎。它通过结构化的决策流程，引导你在创造性工作中做出更好的判断——而不是把判断交给 AI。

## 安装

```bash
# 从源码构建
git clone https://github.com/AltriaSuki/process-cli.git
cd process-cli
cargo install --path .
```

构建完成后，`process-cli` 会安装到 `~/.cargo/bin/`。建议设置 alias：

```bash
alias process="process-cli"
```

### Shell 补全

```bash
# Bash
process-cli completions bash >> ~/.bashrc

# Zsh
process-cli completions zsh >> ~/.zshrc

# Fish
process-cli completions fish > ~/.config/fish/completions/process-cli.fish
```

## Quick Start

### 1. 全新项目

```bash
# 初始化
mkdir my-project && cd my-project
process-cli init

# 编辑 seed.yaml（项目定义）
$EDITOR .process/seed.yaml

# 验证 seed
process-cli seed-validate

# 查看 AI 配置
process-cli ai-config show
```

### 2. 决策流程

```bash
# Phase 1: 发散 — 生成多个架构方案
process-cli diverge
process-cli diverge-validate
process-cli diverge-challenge    # 挑战方案，逼自己深度思考

# Phase 2: 收敛 — 选择一个方向，提取规则
process-cli converge
process-cli converge-validate
process-cli converge-challenge   # 挑战选择，确认决策质量

# Phase 3: 骨架 — 生成项目结构
process-cli skeleton
process-cli skeleton-validate
```

### 3. 分支工作流

```bash
# Phase 4: 分支循环 — 逐个实现功能
process-cli branch new auth-system
process-cli branch start auth-system
process-cli branch review auth-system          # 4 角色 AI 审查
process-cli branch review auth-system -r security  # 仅安全审查
process-cli branch abuse auth-system           # 对抗性测试
process-cli branch gate auth-system            # 合并门检查
process-cli branch merge auth-system
```

### 4. 收尾

```bash
# Phase 5: 稳定 — 冻结不变量
process-cli stabilize

# Phase 6: 复盘 — AI 生成决策回顾
process-cli postmortem

# Phase 7: 完成
process-cli done
```

### 5. 已有项目接入

```bash
# 对已有项目运行 adopt 扫描
cd existing-project
process-cli init
process-cli adopt all
```

单独运行某个 adopt pass：

```bash
process-cli adopt scan-structure      # 检测语言/框架/目录
process-cli adopt scan-dependencies   # 解析依赖清单
process-cli adopt infer-conventions   # 推断编码规范 (AI)
process-cli adopt scan-git-history    # 从 git 提取决策 (AI)
process-cli adopt gap-analysis        # 识别缺失的决策记录 (AI)
```

## AI 配置

支持 5 个 AI Provider，按优先级自动选择：

| Provider | 优先级 | 环境变量 |
|----------|--------|----------|
| Claude CLI | 95 | 需安装 `claude` 命令 |
| Claude API | 90 | `ANTHROPIC_API_KEY` |
| OpenAI | 80 | `OPENAI_API_KEY` |
| Ollama | 30 | 本地运行 `ollama serve` |
| Manual | 1 | 无需配置，手动粘贴 |

```bash
# 查看当前配置和可用 provider
process-cli ai-config show

# 测试连接
process-cli ai-config test

# 手动指定 provider
process-cli ai-config set-provider openai
```

## 决策流程图

```
Init → Diverge → Converge → Skeleton → Branching → Stabilize → Postmortem → Done
       (发散)    (收敛)      (骨架)     (分支循环)   (稳定)      (复盘)       (完成)
```

每个阶段只能前进，不能后退。这是刻意的设计——做了决定就要承担后果。

## 自动化工具

### Generators — 生成项目文件

```bash
process-cli generate git-hooks   # pre-commit / pre-push
process-cli generate cicd        # GitHub Actions workflow
process-cli generate makefile    # 标准 Makefile targets
process-cli generate ide         # VS Code settings
process-cli generate all         # 全部生成
```

### Checks — 自动化检查

```bash
process-cli check sensitive      # 扫描 API key / 密钥泄露
process-cli check todo           # 扫描 TODO/FIXME
process-cli check lint           # 运行 linter
process-cli check test           # 运行测试
process-cli check all            # 全部检查
```

## 项目文件结构

所有状态保存在 `.process/` 目录下（建议 git 跟踪）：

```
.process/
├── .state.yaml              # 当前 Phase 状态
├── config.yaml              # 项目级配置
├── seed.yaml                # 项目定义（6 字段）
├── diverge_summary.yaml     # 发散阶段输出
├── converge_summary.yaml    # 收敛阶段输出（规则）
├── skeleton.yaml            # 骨架定义
├── decisions_log.yaml       # 决策日志
├── manifest.yaml            # Pass Engine artifact 注册表
├── learnings.yaml           # 学习记录
├── frictions.yaml           # 摩擦点记录
└── branches/                # 分支假设 + 审查结果
    ├── auth-system.yaml
    └── auth-system-review.yaml
```

## 自定义 Prompt 模板

在 `.process/prompts/` 下放置 `.md.tera` 文件即可覆盖内置模板：

```
.process/prompts/
├── _default/                # 所有 provider 通用
│   └── diverge.md.tera
└── claude/                  # Claude 专用（XML 标签风格）
    └── diverge.md.tera
```

查找优先级：项目 provider → 项目 default → 内置 provider → 内置 default。

## 常用命令速查

```bash
process-cli status               # 查看当前状态
process-cli guide                # 按类别查看所有命令
process-cli learn "教训内容"      # 记录学习
process-cli friction feat "描述"  # 记录摩擦点
process-cli pass list            # 列出所有 Pass
process-cli pass run-all         # 按依赖顺序运行所有 Pass
```

## License

MIT
