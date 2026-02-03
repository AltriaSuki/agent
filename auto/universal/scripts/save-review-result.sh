#!/bin/bash
# 保存 Opus 的审查结果

REVIEW_DIR="${HOME}/.code-reviews"
LAST_REVIEW_FILE="${REVIEW_DIR}/.last_review"

if [ ! -f "$LAST_REVIEW_FILE" ]; then
    echo "❌ 没有找到上次的审查请求"
    exit 1
fi

REVIEW_FILE=$(cat "$LAST_REVIEW_FILE")
RESULT_FILE="${REVIEW_FILE%.md}_result.md"

echo "📝 请粘贴 Opus 的审查结果（输入 END 结束）："
echo ""

# 读取多行输入
RESULT=""
while IFS= read -r line; do
    if [ "$line" = "END" ]; then
        break
    fi
    RESULT="${RESULT}${line}\n"
done

# 保存结果
echo -e "$RESULT" > "$RESULT_FILE"

echo ""
echo "✅ 审查结果已保存到: $RESULT_FILE"
echo ""

# 解析 Critical 和 Major 问题
CRITICAL_COUNT=$(grep -c "Critical" "$RESULT_FILE" 2>/dev/null || echo "0")
MAJOR_COUNT=$(grep -c "Major" "$RESULT_FILE" 2>/dev/null || echo "0")

if [ "$CRITICAL_COUNT" -gt 0 ] || [ "$MAJOR_COUNT" -gt 0 ]; then
    echo "⚠️  发现问题："
    echo "  - Critical: $CRITICAL_COUNT"
    echo "  - Major: $MAJOR_COUNT"
    echo ""
    echo "建议修复后再提交代码"
else
    echo "✅ 没有发现严重问题，可以提交！"
fi
