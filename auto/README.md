# 自动化开发工具集

这个目录包含了所有自动化开发工具和配置文件。

## 📁 目录结构

```
auto/
├── 1e-project/              # 特定于本项目的配置
│   ├── githooks/            # Git hooks（代码提交检查）
│   │   ├── pre-commit       # 提交前检查
│   │   └── pre-push         # 推送前检查
│   ├── scripts/
│   │   └── setup-hooks.sh   # 安装 Git hooks
│   ├── github-workflows/    # GitHub Actions CI/CD
│   │   ├── ci.yml           # 持续集成
│   │   └── auto-review.yml  # 自动代码审查
│   ├── vscode/              # VS Code 配置
│   │   ├── settings.json    # 编辑器设置
│   │   ├── tasks.json       # 任务定义
│   │   └── extensions.json  # 推荐插件
│   ├── Makefile             # 开发命令集合
│   ├── DEVELOPMENT.md       # 开发指南
│   └── ARCHITECTURE_DESIGN.md  # 技术架构设计
│
└── universal/               # 通用工具（适用于所有项目）
    ├── claude/
    │   ├── workflows/
    │   │   └── code-and-review.md  # Claude Code 工作流
    │   ├── AUTO_CODE_REVIEW.md     # 自动审查完整文档
    │   └── QUICK_START.md          # 快速开始指南
    ├── scripts/
    │   ├── code-review-workflow.sh    # 半自动审查流程
    │   ├── prepare-review.sh          # 准备审查（推荐）
    │   ├── save-review-result.sh      # 保存审查结果
    │   └── BUDGET_WORKFLOW.md         # 经济实惠工作流
    └── README.md                      # 本文件
```

## 🚀 快速开始

### 方案选择

根据你的需求选择：

#### 1️⃣ **经济实惠方案**（推荐）
适合：经济拮据，主要用免费模型开发

```bash
# 查看文档
cat universal/scripts/BUDGET_WORKFLOW.md

# 使用工具
cd universal/scripts
chmod +x prepare-review.sh
./prepare-review.sh
```

💰 **省钱原理**：
- 日常写代码用 Antigravity（免费）
- 只在需要深度审查时用 Claude Code Opus
- 节省 70-80% 成本

#### 2️⃣ **Claude Code 全自动方案**
适合：不差钱，想要完全自动化

```bash
# 查看快速开始
cat universal/claude/QUICK_START.md

# 在 Claude Code 中直接说：
"实现 XXX 功能，完成后自动用 Opus 审查"
```

#### 3️⃣ **本项目专用方案**
适合：为本项目（日省录）配置完整的 CI/CD

```bash
# 安装 Git hooks
bash 1e-project/scripts/setup-hooks.sh

# 使用 Makefile 命令
cd ..  # 回到项目根目录
make help
make check  # 运行所有检查
```

## 📚 文档导航

### 新手入门
1. **先看**: `universal/claude/QUICK_START.md` - 最简单的用法
2. **省钱**: `universal/scripts/BUDGET_WORKFLOW.md` - 经济实惠方案
3. **深入**: `universal/claude/AUTO_CODE_REVIEW.md` - 完整文档

### 项目开发者
1. **开发指南**: `1e-project/DEVELOPMENT.md` - 本项目开发流程
2. **架构设计**: `1e-project/ARCHITECTURE_DESIGN.md` - 技术架构改进方案
3. **CI/CD**: `1e-project/github-workflows/` - 自动化测试和部署

## 🎯 使用建议

### 如果你经济拮据（推荐）

```bash
# 1. 设置工具
cd universal/scripts
chmod +x *.sh

# 2. 日常开发用免费模型（Antigravity 等）

# 3. 完成功能后
./prepare-review.sh

# 4. 打开 Claude Code，切换到 Opus，粘贴审查

# 5. 根据审查意见，继续用免费模型修复
```

**成本**: 每次审查约 300-500 tokens（Opus）

### 如果你不差钱

在 Claude Code 中直接对话：
```
实现登录功能，完成后自动用 Opus 深度审查，发现严重问题自动修复
```

**成本**: 每次完整流程约 1000-1500 tokens

## 🔧 安装与配置

### 通用工具安装

```bash
# 复制到你的工具目录
cp universal/scripts/*.sh ~/.local/bin/

# 添加执行权限
chmod +x ~/.local/bin/*.sh

# 确保在 PATH 中
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 本项目配置

```bash
# 1. 安装 Git hooks
bash 1e-project/scripts/setup-hooks.sh

# 2. 复制 VS Code 配置（可选）
cp -r 1e-project/vscode/* .vscode/

# 3. 复制 Makefile 到项目根目录
cp 1e-project/Makefile ../

# 4. 复制 GitHub workflows（如果需要 CI/CD）
mkdir -p ../.github/workflows
cp 1e-project/github-workflows/* ../.github/workflows/
```

## 💡 常见问题

### Q: 这些工具都要用吗？

**A**: 不用！根据需求选择：
- 经济拮据 → 只用 `universal/scripts/prepare-review.sh`
- 需要本项目 CI/CD → 用 `1e-project/` 下的配置
- 想要完全自动化 → 参考 `universal/claude/` 文档

### Q: 会不会太复杂？

**A**: 最简单的用法只需要：
```bash
prepare-review.sh  # 收集代码变更
# 然后粘贴到 Claude Code Opus 审查
```

### Q: 我能用于其他项目吗？

**A**: 可以！
- `universal/` 下的所有工具都是通用的
- `1e-project/` 下的配置可以作为模板修改

## 📊 效果对比

### 传统方式
```
写代码 → 切换到 Opus → 复制代码 → 等待审查 → 复制审查意见 → 切回去修复
耗时: ~20 分钟
成本: 1000+ tokens
```

### 自动化方式（经济版）
```
写代码（Antigravity 免费） → prepare-review.sh → 粘贴到 Opus → 修复（Antigravity）
耗时: ~5 分钟
成本: 300-500 tokens（省 70%）
```

## 🎓 学习路径

### Day 1: 基础
- 阅读 `universal/claude/QUICK_START.md`
- 试用 `prepare-review.sh` 工具

### Day 2: 进阶
- 阅读 `universal/scripts/BUDGET_WORKFLOW.md`
- 学习审查模板使用

### Day 3: 高级
- 配置 Git hooks（自动检查）
- 设置 VS Code 集成

### Week 2: 优化
- 根据实际使用调整工作流
- 创建自己的审查模板

## 🔗 相关资源

- [Claude Code 文档](https://docs.anthropic.com/claude/docs)
- [Git Hooks 教程](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
- [GitHub Actions](https://docs.github.com/en/actions)

## 🆘 获取帮助

- 查看各个文档的详细说明
- 在 Claude Code 中直接问："如何使用自动化审查工具？"

---

**推荐起点**:
1. 经济实惠 → `universal/scripts/BUDGET_WORKFLOW.md`
2. 完全自动 → `universal/claude/QUICK_START.md`
3. 本项目开发 → `1e-project/DEVELOPMENT.md`
