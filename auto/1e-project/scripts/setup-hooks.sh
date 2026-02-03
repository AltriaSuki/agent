#!/bin/bash
# 安装 Git Hooks

echo "📦 安装 Git Hooks..."

# 创建 hooks 目录（如果不存在）
mkdir -p .git/hooks

# 复制 hooks
cp .githooks/pre-commit .git/hooks/pre-commit
cp .githooks/pre-push .git/hooks/pre-push

# 添加执行权限
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push

echo "✅ Git Hooks 安装完成！"
echo ""
echo "现在每次 commit 和 push 都会自动运行检查："
echo "  • 代码格式化 (rustfmt)"
echo "  • 代码质量检查 (clippy)"
echo "  • 单元测试"
echo "  • 敏感信息检查"
echo ""
echo "如需跳过检查，使用: git commit --no-verify"
