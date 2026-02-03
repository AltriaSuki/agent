# 开发指南 - 自动化工作流

> 让机器帮你检查代码，你只管写

## 🚀 快速开始

### 1. 一键安装所有工具

```bash
# 安装 Git Hooks（自动检查）
make install-hooks

# 验证安装
git config core.hooksPath
```

现在每次 `git commit` 都会自动：
- ✅ 格式化代码
- ✅ 运行 Clippy 检查
- ✅ 运行测试
- ✅ 检查敏感信息

### 2. 日常开发流程

#### 写代码前
```bash
# 拉取最新代码
git pull

# 创建功能分支
git checkout -b feature/your-feature
```

#### 写代码时（VS Code 自动处理）
- **保存时自动格式化** - 不需要手动运行 `cargo fmt`
- **实时代码提示** - Rust Analyzer 自动检查
- **错误高亮** - Clippy 问题实时显示

#### 提交前（自动运行，无需手动）
```bash
git add .
git commit -m "feat: 你的功能"

# Git hook 自动运行：
# ✅ cargo fmt --check
# ✅ cargo clippy
# ✅ cargo test
# ✅ 敏感信息检查
```

如果检查失败，**代码会自动格式化**，你只需重新 add 即可：
```bash
git add .
git commit -m "feat: 你的功能"  # 再次提交
```

#### Push 前（自动运行）
```bash
git push

# 自动运行完整测试套件
# 如果失败会阻止 push
```

## 🛠️ 开发命令速查

### 最常用（推荐）

```bash
# 提交前快速检查（2秒内完成）
make quick-check

# 完整检查（等同于 CI，推荐 push 前使用）
make check

# 启动开发服务器
make dev
```

### 其他常用命令

```bash
# 自动修复所有能自动修复的问题
make fix

# 运行测试
make test

# 监听文件变化，自动运行测试
make watch

# 构建 Android APK
make apk

# 查看所有命令
make help
```

## 🤖 自动化工具说明

### 1. Git Hooks（本地自动检查）

**Pre-commit Hook**（提交时运行）
- 代码格式化检查
- Clippy 代码质量检查
- 单元测试
- 敏感信息扫描

**Pre-push Hook**（推送时运行）
- 完整测试套件
- 系统测试
- 未提交更改检查

### 2. GitHub Actions（云端 CI）

每次 push 或 PR 都会自动运行：
- ✅ 代码格式检查
- ✅ Clippy 检查
- ✅ 多平台测试（Ubuntu + macOS）
- ✅ 代码覆盖率
- ✅ 安全审计
- ✅ Android 构建测试

**查看 CI 状态**: 在 GitHub PR 页面查看

### 3. Makefile（统一命令入口）

所有开发命令都封装在 Makefile 中：
```bash
make <命令>
```

### 4. VS Code 配置（编辑器集成）

**自动化功能**：
- 保存时自动格式化
- 实时 Clippy 检查
- 一键运行任务（`Cmd+Shift+B`）
- 代码补全和重构

**推荐插件**：
打开项目后会提示安装，一键安装即可。

## 📝 工作流示例

### 场景 1: 添加新功能

```bash
# 1. 创建分支
git checkout -b feature/add-search

# 2. 写代码（VS Code 自动格式化和检查）

# 3. 提交（自动运行检查）
git add .
git commit -m "feat: add search functionality"

# 如果检查失败，会自动修复格式问题
# 只需重新 add 并 commit

# 4. Push（自动运行完整测试）
git push origin feature/add-search

# 5. 创建 PR（GitHub Actions 自动运行 CI）
```

### 场景 2: 修复 Bug

```bash
# 1. 快速检查代码质量
make quick-check

# 2. 修复代码

# 3. 运行测试
make test

# 4. 提交（自动检查）
git commit -am "fix: resolve data corruption issue"

# 5. Push
git push
```

### 场景 3: 重构代码

```bash
# 1. 先确保测试通过
make test

# 2. 重构代码

# 3. 自动修复格式和简单问题
make fix

# 4. 完整检查
make check

# 5. 提交
git commit -am "refactor: improve code organization"
```

## 🎯 最佳实践

### ✅ DO (推荐)

1. **依赖自动化工具**
   - 让 Git hooks 帮你检查
   - 不要手动运行 `cargo fmt`（保存时自动）
   - 不要手动运行 `cargo test`（commit 时自动）

2. **提交前快速验证**
   ```bash
   make quick-check  # 2秒内完成
   ```

3. **Push 前完整检查**
   ```bash
   make check  # 确保通过 CI
   ```

4. **利用 VS Code 集成**
   - 使用 `Cmd+Shift+B` 快速运行任务
   - 查看 "问题" 面板看 Clippy 警告

5. **遇到问题先自动修复**
   ```bash
   make fix
   ```

### ❌ DON'T (避免)

1. **不要跳过 hooks**
   ```bash
   git commit --no-verify  # ⚠️ 只在紧急情况使用
   ```

2. **不要忽略 CI 失败**
   - 如果 CI 失败，本地运行 `make check` 调试

3. **不要手动格式化代码**
   - VS Code 保存时自动格式化
   - 或使用 `make fmt`

4. **不要在主分支直接提交**
   - 永远使用功能分支
   - 通过 PR 合并

## 🔧 故障排查

### 问题：Git hook 没有运行

```bash
# 重新安装 hooks
make install-hooks

# 检查权限
ls -la .git/hooks/pre-commit
# 应该显示 -rwxr-xr-x（可执行）

# 如果没有执行权限
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
```

### 问题：Clippy 检查失败

```bash
# 自动修复
make fix

# 手动修复后重新检查
make lint
```

### 问题：测试失败

```bash
# 查看详细错误
cargo test -- --nocapture

# 只运行特定测试
cargo test test_name
```

### 问题：VS Code 没有自动格式化

1. 检查是否安装了 `rust-analyzer` 插件
2. 打开设置，搜索 "format on save"，确保启用
3. 重启 VS Code

## 📊 CI/CD 状态监控

### 查看构建状态

1. 访问 GitHub Actions 页面
2. 查看最新的 workflow 运行
3. 点击失败的 job 查看详细日志

### 本地模拟 CI

```bash
# 运行完整的 CI 检查
make full-check

# 这会运行：
# - 格式检查
# - Clippy 检查
# - 测试
# - 安全审计
# - 代码覆盖率
```

## 🎓 进阶技巧

### 1. 自定义 Git Hook

编辑 `.githooks/pre-commit` 添加自定义检查：

```bash
# 例如：检查 commit message 格式
if ! grep -qE "^(feat|fix|docs|style|refactor|test|chore):" <<< "$commit_msg"; then
    echo "❌ Commit message 格式错误"
    exit 1
fi
```

### 2. 添加自定义 Makefile 命令

```makefile
# 添加到 Makefile
my-command:
    @echo "运行自定义命令"
    @cargo build --features my-feature
```

### 3. 配置 Watch 模式

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 监听代码变化，自动运行测试
make watch
```

## 📚 相关文档

- [ARCHITECTURE_DESIGN.md](./ARCHITECTURE_DESIGN.md) - 技术架构设计
- [DESIGN.md](./DESIGN.md) - 产品设计
- [README.md](./README.md) - 项目介绍

## 🆘 获取帮助

遇到问题？

1. 查看本文档的故障排查部分
2. 运行 `make help` 查看所有命令
3. 查看 GitHub Issues
4. 联系团队成员

---

**记住**: 自动化工具是为了让你专注于写代码，不是为了增加负担。如果某个检查阻碍了你的开发，可以临时跳过（`--no-verify`），但要在 PR 前修复！
