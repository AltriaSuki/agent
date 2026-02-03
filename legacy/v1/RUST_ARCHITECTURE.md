# Process CLI - Rust 重写方案

## 概述

将 1,519 行的 `process-cli.sh` 重写为可扩展的 Rust CLI 工具，整合 `auto/` 文件夹中的所有功能。

## 架构设计原则

1. **Trait-based 扩展** - 通过 trait 定义扩展点，新增功能无需修改核心代码
2. **Registry 模式** - 所有可扩展组件使用注册表，支持运行时发现
3. **配置驱动** - 多层配置（默认值 → 全局 → 项目 → 环境变量）
4. **Workspace 结构** - 分离关注点，独立测试和演进

---

## 项目结构

```
process-cli/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── process-core/             # 核心领域逻辑（Phase、Branch、State）
│   ├── process-ai/               # AI Provider 抽象
│   ├── process-generators/       # 代码/配置生成器
│   ├── process-checks/           # 自动化检查
│   ├── process-reviews/          # Review 模板
│   └── process-config/           # 配置系统
├── src/                          # 主 CLI 二进制
│   ├── main.rs
│   ├── cli.rs                    # Clap 定义
│   ├── commands/                 # 命令实现
│   └── output.rs                 # 输出格式化
└── templates/                    # 模板文件
    ├── prompts/                  # AI 提示词
    └── generators/               # 生成器模板
```

---

## 核心 Traits（扩展点）

### 1. Command Trait
```rust
#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &[&'static str] { &[] }
    fn required_phase(&self) -> Option<PhaseRequirement>;
    async fn execute(&self, ctx: &AppContext, args: &CommandArgs) -> Result<()>;
}
```

### 2. AiProvider Trait
```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u8;  // 自动检测优先级
    async fn is_available(&self) -> bool;
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;
}
```

### 3. Generator Trait
```rust
pub trait Generator: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;  // git, ci, ide, etc.
    fn generate(&self, ctx: &GeneratorContext) -> Result<GeneratorOutput>;
    fn is_applicable(&self, project: &ProjectInfo) -> bool;
}
```

### 4. Check Trait
```rust
#[async_trait]
pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> CheckCategory;
    fn severity(&self) -> Severity;
    async fn run(&self, ctx: &CheckContext) -> Result<CheckResult>;
    fn can_auto_fix(&self) -> bool { false }
    async fn auto_fix(&self, issues: &[Issue]) -> Result<FixResult>;
}
```

### 5. ReviewTemplate Trait
```rust
pub trait ReviewTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn perspectives(&self) -> &[ReviewPerspective];
    fn build_prompt(&self, ctx: &ReviewContext) -> Result<String>;
    fn parse_response(&self, response: &str) -> Result<ReviewResult>;
}
```

---

## 命令列表

### 原有命令（从 bash 迁移）
| 命令 | 阶段 | 说明 |
|------|------|------|
| `init` | 0 | 初始化项目 seed |
| `diverge` | 1 | 多模型发散探索 |
| `converge` | 2 | 收敛决策，提取规则 |
| `skeleton` | 3 | 生成项目骨架 |
| `branch new/start/review/abuse/gate/merge` | 4 | 分支工作流 |
| `stabilize` | 5 | 进入稳定阶段 |
| `postmortem` / `done` | 6 | 回顾总结 |
| `status` | - | 显示当前状态 |
| `learn` / `friction` | - | 记录学习/摩擦点 |
| `ai-config` | - | AI 配置 |

### 新增命令
| 命令 | 说明 |
|------|------|
| `generate git-hooks` | 生成 pre-commit/pre-push |
| `generate ci --target github\|gitlab` | 生成 CI/CD 配置 |
| `generate makefile` | 生成 Makefile |
| `generate ide --target vscode` | 生成 IDE 配置 |
| `generate all` | 生成所有适用配置 |
| `check lint` | 运行 lint 检查 |
| `check test` | 运行测试 |
| `check sensitive` | 敏感信息检测 |
| `check todo` | TODO/FIXME 检测 |
| `check audit` | 依赖安全审计 |
| `check coverage` | 代码覆盖率 |
| `check all` | 运行所有检查 |
| `branch review --template security\|performance\|architecture` | 专项审查 |

---

## AI Providers

| Provider | 优先级 | 检测方式 |
|----------|--------|----------|
| Claude CLI | 100 | `which claude` |
| Claude API | 90 | `ANTHROPIC_API_KEY` |
| OpenAI | 80 | `OPENAI_API_KEY` |
| Ollama | 70 | `http://localhost:11434` |
| Manual | 0 | 兜底，复制到剪贴板 |

---

## Generators（生成器）

### GitHooksGenerator
- 输出: `.git/hooks/pre-commit`, `.git/hooks/pre-push`
- 功能: fmt, lint, test, 敏感信息检测, TODO 检测
- 模板: Tera 模板，支持多语言（Rust/Python/Node）

### CiCdGenerator
- 目标: GitHub Actions, GitLab CI, CircleCI
- 功能: check, test, coverage, audit, benchmark
- 模板: 根据项目语言和配置生成

### MakefileGenerator
- 输出: `Makefile`
- 命令: check, test, fmt, lint, build, clean, etc.

### IdeGenerator
- 目标: VS Code, IntelliJ
- 输出: settings.json, tasks.json, extensions.json

---

## Checks（检查器）

| Check | 类别 | 严重性 | 自动修复 |
|-------|------|--------|----------|
| LintCheck | Lint | Error | 部分 |
| TestCheck | Test | Error | No |
| SensitiveInfoCheck | Security | Critical | No |
| TodoCheck | Style | Info | No |
| SecurityAuditCheck | Security | Warning | No |
| CoverageCheck | Test | Warning | No |

### SensitiveInfoCheck 检测模式
```
password|passwd|pwd = ...
api[_-]?key = ...
secret|token = ...
aws[_-]?(access[_-]?key|secret)
-----BEGIN PRIVATE KEY-----
jdbc://user:pass@host
```

---

## Review Templates（审查模板）

### GeneralReviewTemplate（默认）
4 个角色：Security Auditor, Performance Engineer, User Advocate, Maintainability Expert

### SecurityReviewTemplate
重点: 注入攻击, 认证授权, 数据保护, CSRF, 速率限制

### PerformanceReviewTemplate
重点: N+1 查询, 算法复杂度, 缓存策略, 内存泄漏

### ArchitectureReviewTemplate
重点: SOLID 原则, 模块划分, 耦合度, 可测试性

---

## 配置系统

### 配置文件位置（优先级从低到高）
1. 内置默认值
2. `~/.config/process-cli/config.yaml`（全局）
3. `.process/config.yaml`（项目）
4. 环境变量 `PROCESS_CLI_*`

### 配置结构示例
```yaml
ai:
  provider: auto  # auto|claude|openai|ollama|manual
  claude:
    model: claude-sonnet-4-20250514
  timeout_secs: 120

generators:
  git_hooks:
    pre_commit:
      run_fmt: true
      run_lint: true
      run_tests: true
      check_sensitive: true
      check_todo: true

checks:
  gate_checks:
    - lint
    - test
    - sensitive
    - audit
  fail_threshold: error  # info|warning|error|critical

reviews:
  default_template: general
```

---

## 扩展示例

### 添加新 AI Provider
```rust
// 1. 实现 AiProvider trait
pub struct MyProvider { ... }
impl AiProvider for MyProvider { ... }

// 2. 注册到 registry
registry.register(Arc::new(MyProvider::new()));
```

### 添加新 Generator
```rust
// 1. 实现 Generator trait
pub struct DockerGenerator;
impl Generator for DockerGenerator { ... }

// 2. 添加模板文件 templates/docker/Dockerfile
// 3. 注册 + 添加 CLI 子命令
```

### 添加新 Check
```rust
// 1. 实现 Check trait
pub struct MyCheck;
impl Check for MyCheck { ... }

// 2. 注册到 CheckRegistry
// 3. 可选：添加到默认 gate_checks
```

---

## 依赖项

```toml
[workspace.dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
reqwest = { version = "0.12", features = ["json"] }
anyhow = "1"
tera = "1.19"
regex = "1"
chrono = { version = "0.4", features = ["serde"] }
colored = "2"
dirs = "5"
```

---

## 实施步骤

### Phase 1: 基础框架
1. 创建 workspace 结构
2. 实现 CLI 框架 (clap)
3. 实现配置系统
4. 实现状态管理 (Phase state machine)

### Phase 2: 核心命令迁移
5. 迁移 init/status 命令
6. 迁移 Phase 1-3 命令 (diverge/converge/skeleton)
7. 迁移 Phase 4 分支命令
8. 迁移 Phase 5-6 命令

### Phase 3: AI 系统
9. 实现 AiProvider trait 和 registry
10. 实现 Claude/OpenAI/Ollama/Manual providers
11. 实现自动检测逻辑

### Phase 4: Generators
12. 实现 Generator trait 和模板系统
13. 实现 GitHooksGenerator
14. 实现 CiCdGenerator
15. 实现 MakefileGenerator
16. 实现 IdeGenerator

### Phase 5: Checks
17. 实现 Check trait 和 registry
18. 实现 LintCheck, TestCheck
19. 实现 SensitiveInfoCheck, TodoCheck
20. 实现 SecurityAuditCheck, CoverageCheck
21. 集成到 branch-gate

### Phase 6: Reviews
22. 实现 ReviewTemplate trait
23. 实现 General/Security/Performance/Architecture 模板
24. 集成到 branch-review

### Phase 7: 完善
25. 添加测试
26. 添加文档
27. 发布

---

## 验证方法

1. **单元测试**: 每个 crate 有独立的测试
2. **集成测试**: 完整工作流测试
3. **手动验证**:
   - `process init` → 检查 .process/ 目录创建
   - `process generate git-hooks` → 检查 .git/hooks/ 生成
   - `process generate ci` → 检查 .github/workflows/ 生成
   - `process check all` → 运行所有检查
   - `process branch-gate <name>` → 验证门禁检查

---

## 参考文件

- `process-cli.sh` - 原始 bash 实现
- `auto/1e-project/githooks/pre-commit` - Git hooks 参考
- `auto/1e-project/github-workflows/ci.yml` - CI 参考
- `auto/universal/scripts/prepare-review.sh` - Review 模板参考
