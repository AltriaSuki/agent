#!/bin/bash
# 准备代码审查 - 收集改动，只在需要时用 Opus 审查

set -e

REVIEW_DIR="${HOME}/.code-reviews"
TEMPLATE_DIR="${HOME}/.code-review-templates"
mkdir -p "$REVIEW_DIR"
mkdir -p "$TEMPLATE_DIR"

# 颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 解析参数
TEMPLATE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --template|-t)
            TEMPLATE="$2"
            shift 2
            ;;
        --help|-h)
            echo "用法: prepare-review.sh [选项]"
            echo ""
            echo "选项:"
            echo "  -t, --template NAME   使用指定的审查模板"
            echo "  -h, --help           显示帮助"
            echo ""
            echo "示例:"
            echo "  prepare-review.sh                    # 标准审查"
            echo "  prepare-review.sh -t security       # 安全审查"
            echo "  prepare-review.sh -t performance    # 性能审查"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}📝 准备代码审查${NC}"

# 检查是否在 git 仓库
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  不在 git 仓库中，将收集所有修改的文件${NC}"
    USE_GIT=false
else
    USE_GIT=true
fi

# 生成审查文件名
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REVIEW_FILE="${REVIEW_DIR}/review_${TIMESTAMP}.md"

# 收集代码变更
echo -e "${GREEN}收集代码变更...${NC}"

cat > "$REVIEW_FILE" << 'EOF'
# 代码审查请求

## 任务描述
EOF

# 询问任务描述
echo ""
echo "请简要描述这次改动的目的（按 Enter 结束）："
read -r DESCRIPTION
echo "$DESCRIPTION" >> "$REVIEW_FILE"

cat >> "$REVIEW_FILE" << 'EOF'

## 代码变更

EOF

if [ "$USE_GIT" = true ]; then
    # 使用 git diff
    echo "### Git Diff" >> "$REVIEW_FILE"
    echo '```diff' >> "$REVIEW_FILE"

    # 如果有 staged 的改动，显示 staged
    if ! git diff --cached --quiet; then
        git diff --cached >> "$REVIEW_FILE"
    else
        # 否则显示 unstaged 的改动
        git diff >> "$REVIEW_FILE"
    fi

    echo '```' >> "$REVIEW_FILE"

    # 统计信息
    echo "" >> "$REVIEW_FILE"
    echo "### 变更统计" >> "$REVIEW_FILE"
    echo '```' >> "$REVIEW_FILE"
    git diff --stat >> "$REVIEW_FILE" 2>/dev/null || git diff --cached --stat >> "$REVIEW_FILE"
    echo '```' >> "$REVIEW_FILE"
else
    echo "（非 git 仓库，请手动粘贴代码）" >> "$REVIEW_FILE"
fi

cat >> "$REVIEW_FILE" << 'EOF'

## 审查要求

EOF

# 加载审查模板
if [ -n "$TEMPLATE" ]; then
    TEMPLATE_FILE="${TEMPLATE_DIR}/${TEMPLATE}.md"
    if [ -f "$TEMPLATE_FILE" ]; then
        echo -e "${GREEN}使用模板: $TEMPLATE${NC}"
        cat "$TEMPLATE_FILE" >> "$REVIEW_FILE"
    else
        echo -e "${YELLOW}⚠️  模板不存在: $TEMPLATE_FILE${NC}"
        echo "创建默认模板..."

        # 创建一些默认模板
        cat > "${TEMPLATE_DIR}/security.md" << 'TEMPLATE_EOF'
### 🔒 安全审查重点

请特别关注以下安全问题：

#### 注入攻击
- [ ] SQL 注入（参数化查询？）
- [ ] NoSQL 注入
- [ ] 命令注入
- [ ] XSS（输入是否转义？）
- [ ] LDAP 注入

#### 认证与授权
- [ ] 密码存储（哈希 + salt？）
- [ ] Session 管理（安全 cookie？）
- [ ] 权限检查（是否完善？）
- [ ] JWT token 验证

#### 数据保护
- [ ] 敏感信息加密
- [ ] HTTPS 使用
- [ ] 敏感日志脱敏
- [ ] API key 保护

#### 其他
- [ ] CSRF 防护
- [ ] 速率限制
- [ ] 输入验证
TEMPLATE_EOF

        cat > "${TEMPLATE_DIR}/performance.md" << 'TEMPLATE_EOF'
### ⚡ 性能审查重点

请特别关注以下性能问题：

#### 数据库
- [ ] N+1 查询问题
- [ ] 缺少索引
- [ ] 查询未优化
- [ ] 是否可以用批量操作
- [ ] 连接池配置

#### 算法与数据结构
- [ ] 时间复杂度（O(n²) 以上？）
- [ ] 空间复杂度
- [ ] 不必要的循环
- [ ] 可以用更高效的数据结构？

#### 缓存
- [ ] 是否应该缓存
- [ ] 缓存失效策略
- [ ] 缓存穿透风险

#### 其他
- [ ] 内存泄漏风险
- [ ] 不必要的计算
- [ ] 异步处理机会
- [ ] 分页/限流
TEMPLATE_EOF

        cat > "${TEMPLATE_DIR}/architecture.md" << 'TEMPLATE_EOF'
### 🏗️ 架构审查重点

请特别关注以下架构问题：

#### 设计原则
- [ ] 单一职责原则
- [ ] 开闭原则
- [ ] 依赖倒置
- [ ] 接口隔离

#### 代码组织
- [ ] 模块划分是否合理
- [ ] 耦合度（高耦合？）
- [ ] 代码复用
- [ ] 是否过度设计

#### 可维护性
- [ ] 代码可读性
- [ ] 复杂度（圈复杂度）
- [ ] 技术债务
- [ ] 向后兼容性

#### 可测试性
- [ ] 是否易于测试
- [ ] 依赖注入
- [ ] 模拟/Mock 友好
TEMPLATE_EOF

        echo -e "${GREEN}✅ 已创建默认模板:${NC}"
        echo "  - security.md"
        echo "  - performance.md"
        echo "  - architecture.md"
        echo ""
        echo "使用默认全面审查..."
        TEMPLATE=""
    fi
fi

# 如果没有指定模板，使用标准审查
if [ -z "$TEMPLATE" ]; then
    cat >> "$REVIEW_FILE" << 'EOF'
请审查以下方面：

### 🔒 安全性
- [ ] SQL 注入风险
- [ ] XSS 风险
- [ ] 敏感信息泄露
- [ ] 权限检查

### ⚡ 性能
- [ ] N+1 查询
- [ ] 不必要的循环
- [ ] 内存泄漏风险
- [ ] 算法复杂度

### 🧪 代码质量
- [ ] 代码重复
- [ ] 函数过长/过于复杂
- [ ] 命名是否清晰
- [ ] 错误处理是否完善

### ✅ 测试
- [ ] 是否需要添加测试
- [ ] 边界情况是否覆盖
- [ ] 错误路径是否测试

### 📝 其他
- [ ] 文档/注释是否充分
- [ ] API 兼容性
- [ ] 是否引入技术债务
EOF
fi

cat >> "$REVIEW_FILE" << 'EOF'

## 审查结果

请按以下格式输出：

**总体评分**: [1-5 分]

**Critical Issues** (必须修复):
-

**Major Issues** (建议修复):
-

**Minor Issues** (可选):
-

**优点**:
-

**建议**:
-

EOF

echo ""
echo -e "${GREEN}✅ 审查请求已准备${NC}"
echo -e "文件位置: ${BLUE}$REVIEW_FILE${NC}"
echo ""
echo -e "${YELLOW}下一步：${NC}"
echo "1. 在 Claude Code 中切换到 Opus 模型"
echo "2. 粘贴以下内容："
echo ""
echo -e "${BLUE}─────────────────────────────────────${NC}"
cat "$REVIEW_FILE"
echo -e "${BLUE}─────────────────────────────────────${NC}"
echo ""
echo "或者运行以下命令自动复制到剪贴板："
echo "  cat \"$REVIEW_FILE\" | pbcopy"
echo ""

# 自动复制到剪贴板（macOS）
if command -v pbcopy &> /dev/null; then
    cat "$REVIEW_FILE" | pbcopy
    echo -e "${GREEN}✅ 已自动复制到剪贴板！${NC}"
    echo "直接在 Claude Code 中 Cmd+V 粘贴即可"
fi

# 保存文件路径以便后续使用
echo "$REVIEW_FILE" > "${REVIEW_DIR}/.last_review"
