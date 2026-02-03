# 💰 经济实惠的开发工作流

> Antigravity (免费) 写代码 + Claude Code Opus (仅审查) = 省钱高效

## 🎯 核心理念

- **日常开发**: 用免费的 Antigravity（或其他免费模型）
- **代码审查**: 只在关键时刻用 Claude Code Opus
- **成本控制**: Opus 只用于审查，不用于写代码

## 📋 完整工作流

### Step 1: 用 Antigravity 写代码

在你常用的 Antigravity 界面中：

```
实现一个用户登录功能，包括：
- 密码验证
- Session 管理
- 错误处理
```

Antigravity 给你代码后，在本地实现。

### Step 2: 本地测试

```bash
# 运行基础测试
npm test  # 或 pytest, cargo test 等

# 确保代码能运行
```

### Step 3: 提交前准备审查

```bash
# 运行审查准备脚本
prepare-review.sh
```

脚本会：
1. 收集你的 git diff
2. 自动生成审查请求文档
3. 复制到剪贴板
4. 包含完整的审查检查清单

### Step 4: 用 Claude Code Opus 审查

1. 打开 Claude Code
2. **切换到 Opus 模型** （重要！）
3. 粘贴刚才复制的内容（Cmd+V）
4. 等待审查结果

### Step 5: 处理审查结果

根据 Opus 的审查意见：

- **有 Critical 问题**: 回到 Antigravity 修复
- **有 Major 问题**: 评估是否需要修复
- **只有 Minor 问题**: 可以提交，后续优化

### Step 6: 提交代码

```bash
git add .
git commit -m "feat: implement user login"
```

## 💡 使用技巧

### 技巧 1: 批量审查

不要每写一行就审查，而是：
- 完成一个完整功能模块后
- 或一天工作结束前
- 一次性审查所有改动

**原因**: 减少 Opus 调用次数 = 省钱

### 技巧 2: 智能选择审查时机

**需要用 Opus 审查的情况**:
- ✅ 安全相关代码（认证、支付、加密）
- ✅ 性能关键路径（数据库查询、算法）
- ✅ 架构级改动（重构、新模块）
- ✅ 准备发布前的最终检查

**不需要 Opus 的情况**:
- ❌ 简单的 UI 调整
- ❌ 注释修改
- ❌ 配置文件更新
- ❌ 测试代码（除非很复杂）

### 技巧 3: 使用审查模板

创建 `~/.code-review-templates/` 目录，保存常用的审查要求：

**security.md** (安全审查):
```markdown
请重点审查安全性：
- SQL 注入
- XSS
- CSRF
- 敏感信息泄露
- 权限检查
```

**performance.md** (性能审查):
```markdown
请重点审查性能：
- 数据库查询优化
- N+1 问题
- 缓存使用
- 算法复杂度
```

使用时：
```bash
# 只审查安全性
prepare-review.sh --template security

# 只审查性能
prepare-review.sh --template performance
```

### 技巧 4: 保存审查历史

所有审查结果都保存在 `~/.code-reviews/` 目录：

```
~/.code-reviews/
├── review_20260203_143022.md        # 审查请求
├── review_20260203_143022_result.md  # Opus 审查结果
├── review_20260203_150033.md
└── ...
```

好处：
- 可以回顾之前的审查意见
- 学习常见问题模式
- 团队知识积累

## 🔧 工具安装

```bash
# 1. 给脚本添加执行权限
chmod +x ~/.local/bin/prepare-review.sh
chmod +x ~/.local/bin/save-review-result.sh

# 2. 确保 ~/.local/bin 在 PATH 中
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 3. 测试
prepare-review.sh
```

## 📊 成本对比

### 方案 A: 全部用 Claude Code Opus
```
写代码 (Opus): 500 tokens
审查 (Opus): 300 tokens
修复 (Opus): 200 tokens
─────────────────────────
总计: 1000 tokens
```

### 方案 B: 混合方案（推荐）
```
写代码 (Antigravity): 免费
审查 (Opus): 300 tokens
修复 (Antigravity): 免费
─────────────────────────
总计: 300 tokens (省 70%)
```

### 方案 C: 批量审查（最省）
```
写代码一整天 (Antigravity): 免费
晚上一次性审查 (Opus): 500 tokens
修复 (Antigravity): 免费
─────────────────────────
总计: 500 tokens
但审查质量更高（一次看全局）
```

## 🎓 最佳实践

### ✅ DO (推荐)

1. **用 Antigravity 快速迭代**
   - 实现功能
   - 调试问题
   - 写测试

2. **关键节点用 Opus**
   - 功能完成时
   - 准备提交前
   - 发布前

3. **保持审查请求清晰**
   - 描述改动目的
   - 提供完整 diff
   - 明确审查重点

4. **批量审查**
   - 一天结束时一次性审查
   - 或完成一个大功能后审查

### ❌ DON'T (避免)

1. **不要每次小改动都用 Opus**
   - 浪费钱
   - 效率低

2. **不要省略本地测试**
   - 先确保代码能跑
   - 再用 Opus 审查质量

3. **不要忽略 Opus 的 Critical 问题**
   - 这些问题值得花时间修复
   - 不要为了省钱跳过

## 🔍 实际案例

### 案例 1: 新功能开发

```
09:00 - 12:00  用 Antigravity 实现功能（免费）
12:00 - 13:00  本地测试和调试（免费）
13:00 - 13:10  prepare-review.sh 准备审查
13:10 - 13:15  Claude Code Opus 审查（花费 Opus 额度）
13:15 - 14:00  用 Antigravity 修复问题（免费）
14:00          提交代码

总 Opus 使用: 5 分钟
```

### 案例 2: Bug 修复

```
# 小 bug，不需要审查
用 Antigravity 修复 → 本地测试 → 直接提交

# 复杂 bug，涉及安全或性能
用 Antigravity 修复 → 本地测试 → Opus 审查 → 确认后提交
```

### 案例 3: 重构

```
# 一周的重构工作
周一-周四: 用 Antigravity 分模块重构（免费）
周五: 用 Opus 做一次完整审查（一次 Opus 调用）
     检查整体架构合理性
```

## 🆘 问题排查

### Q: prepare-review.sh 没有复制到剪贴板

**A**: 手动复制：
```bash
cat ~/.code-reviews/.last_review | pbcopy
```

### Q: git diff 太大，超过 Opus 上下文限制

**A**: 分批审查：
```bash
# 只审查特定文件
git diff path/to/file.js | pbcopy
```

### Q: Antigravity 生成的代码质量不好

**A**:
- 给更详细的提示
- 或者这种情况下，直接用 Claude Code Sonnet（比 Opus 便宜）
- 留 Opus 专门做最终审查

## 📈 优化建议

### 1 个月后评估

跟踪：
- Opus 使用频率
- 发现的问题数量
- 修复成本

优化：
- 如果 Opus 发现很多问题 → 改进 Antigravity 的 prompt
- 如果很少发现问题 → 可以减少审查频率

### 建立个人/团队审查标准

根据 Opus 的审查历史，总结常见问题：
- 创建 checklist
- 写代码时主动避免
- 减少 Opus 审查工作量

---

**记住**:
- 💰 Antigravity 负责"量"（快速开发）
- 💎 Opus 负责"质"（深度审查）
- 🎯 合理分工 = 高效 + 省钱

开始你的经济实惠开发之旅！
