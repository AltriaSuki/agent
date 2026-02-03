#!/bin/bash
# 通用的代码编写 + Review 自动化工作流
# 适用于任何项目

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🤖 Code & Review 自动化工作流${NC}"
echo ""

# 检查是否在 git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ 当前目录不是 git 仓库${NC}"
    exit 1
fi

# 获取任务描述
TASK="${1:-}"
if [ -z "$TASK" ]; then
    echo "用法: code-review-workflow.sh \"任务描述\""
    echo ""
    echo "示例:"
    echo "  code-review-workflow.sh \"添加用户认证功能\""
    exit 1
fi

# 创建临时工作分支
BRANCH_NAME="auto-workflow-$(date +%s)"
echo -e "${YELLOW}📋 任务: $TASK${NC}"
echo -e "${BLUE}🌿 创建工作分支: $BRANCH_NAME${NC}"
git checkout -b "$BRANCH_NAME"

# 阶段 1: 使用 Claude 实现功能
echo ""
echo -e "${GREEN}=== 阶段 1: 实现功能 (Sonnet) ===${NC}"
cat > /tmp/claude-implement-prompt.txt << EOF
请实现以下功能：

$TASK

要求：
1. 分析现有代码结构
2. 实现功能
3. 添加必要的测试
4. 保持代码风格一致
5. 在完成后创建一个 IMPLEMENTATION.md 文件，说明：
   - 实现了什么
   - 修改了哪些文件
   - 如何测试
   - 有什么注意事项
EOF

echo "📝 实现提示已准备"
echo ""
echo -e "${YELLOW}⏸️  现在请：${NC}"
echo "1. 在 Claude Code 中执行上述任务"
echo "2. 完成后按 Enter 继续..."
cat /tmp/claude-implement-prompt.txt
read -p ""

# 检查是否有改动
if git diff --quiet; then
    echo -e "${RED}❌ 没有检测到代码改动${NC}"
    git checkout -
    git branch -D "$BRANCH_NAME"
    exit 1
fi

# 提交实现
echo -e "${BLUE}💾 提交实现...${NC}"
git add .
git commit -m "feat: $TASK (implementation)"

# 阶段 2: 代码审查
echo ""
echo -e "${GREEN}=== 阶段 2: 代码审查 (Opus) ===${NC}"

# 生成 diff
git diff HEAD~1 > /tmp/code-changes.diff

cat > /tmp/claude-review-prompt.txt << EOF
请审查以下代码改动：

任务：$TASK

代码 Diff:
\`\`\`diff
$(cat /tmp/code-changes.diff)
\`\`\`

请从以下角度审查：
1. **代码质量**: 是否有代码异味、重复代码、过度复杂的逻辑
2. **架构设计**: 是否符合现有架构模式，是否引入了不必要的耦合
3. **性能**: 是否有性能问题，如 N+1 查询、不必要的计算
4. **安全性**: 是否有安全漏洞，如 SQL 注入、XSS、敏感信息泄露
5. **测试**: 测试覆盖是否充分，边界情况是否考虑
6. **可维护性**: 代码是否易读、易维护，注释是否充分

请生成一个 CODE_REVIEW.md 文件，包含：
- 总体评价（1-5 分）
- 发现的问题（按严重程度分类：Critical/Major/Minor）
- 改进建议
- 优点（值得保持的地方）
EOF

echo "📝 审查提示已准备"
echo ""
echo -e "${YELLOW}⏸️  现在请：${NC}"
echo "1. 切换到 Opus 模型"
echo "2. 在 Claude Code 中执行上述审查任务"
echo "3. 完成后按 Enter 继续..."
cat /tmp/claude-review-prompt.txt
read -p ""

# 检查是否生成了审查报告
if [ ! -f "CODE_REVIEW.md" ]; then
    echo -e "${YELLOW}⚠️  未找到 CODE_REVIEW.md，可能审查还未完成${NC}"
    echo "是否继续？(y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "工作流中断"
        exit 1
    fi
fi

# 阶段 3: 根据审查意见优化
echo ""
echo -e "${GREEN}=== 阶段 3: 根据审查优化 (Sonnet) ===${NC}"

if [ -f "CODE_REVIEW.md" ]; then
    CRITICAL_ISSUES=$(grep -c "Critical" CODE_REVIEW.md || echo "0")
    MAJOR_ISSUES=$(grep -c "Major" CODE_REVIEW.md || echo "0")

    echo -e "发现问题："
    echo -e "  - Critical: $CRITICAL_ISSUES"
    echo -e "  - Major: $MAJOR_ISSUES"

    if [ "$CRITICAL_ISSUES" -gt 0 ] || [ "$MAJOR_ISSUES" -gt 0 ]; then
        cat > /tmp/claude-fix-prompt.txt << EOF
请根据以下代码审查报告修复问题：

$(cat CODE_REVIEW.md)

要求：
1. 优先修复所有 Critical 问题
2. 尽量修复 Major 问题
3. 更新测试以验证修复
4. 在 FIXES.md 中记录修复了哪些问题
EOF

        echo "📝 优化提示已准备"
        echo ""
        echo -e "${YELLOW}⏸️  现在请：${NC}"
        echo "1. 切换回 Sonnet 模型"
        echo "2. 在 Claude Code 中执行上述优化任务"
        echo "3. 完成后按 Enter 继续..."
        cat /tmp/claude-fix-prompt.txt
        read -p ""

        # 提交修复
        if ! git diff --quiet; then
            git add .
            git commit -m "fix: address code review issues"
        fi
    else
        echo -e "${GREEN}✅ 没有严重问题，无需修复${NC}"
    fi
fi

# 阶段 4: 总结
echo ""
echo -e "${GREEN}=== 工作流完成 ===${NC}"
echo ""
echo "📊 总结："
echo "  - 工作分支: $BRANCH_NAME"
echo "  - 提交数: $(git rev-list --count HEAD ^main 2>/dev/null || echo '?')"
echo ""
echo "🎯 下一步操作："
echo "  1. 查看改动: git diff main"
echo "  2. 运行测试: make test (或项目的测试命令)"
echo "  3. 合并到主分支: git checkout main && git merge $BRANCH_NAME"
echo "  4. 删除工作分支: git branch -d $BRANCH_NAME"
echo ""
echo "或者创建 PR:"
echo "  git push origin $BRANCH_NAME"
echo "  gh pr create --title \"$TASK\""
echo ""

# 清理临时文件
rm -f /tmp/claude-*.txt /tmp/code-changes.diff
