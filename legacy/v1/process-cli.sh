#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# process-cli.sh — 自动化开发流程引擎
# 用法: ./process-cli.sh <command> [args]
# ============================================================

PROCESS_DIR=".process"
SEED_FILE="$PROCESS_DIR/seed.yaml"
DIVERGE_FILE="$PROCESS_DIR/diverge_summary.yaml"
RULES_FILE="$PROCESS_DIR/rules.yaml"
DECISIONS_FILE="$PROCESS_DIR/decisions_log.yaml"
SKELETON_FILE="$PROCESS_DIR/skeleton.yaml"
BRANCHES_DIR="$PROCESS_DIR/branches"
FRICTION_FILE="$PROCESS_DIR/friction.yaml"
LEARNINGS_FILE="$PROCESS_DIR/learnings.yaml"
POSTMORTEM_FILE="$PROCESS_DIR/postmortem.yaml"
REJECTED_FILE="$PROCESS_DIR/REJECTED_APPROACHES.md"
STATE_FILE="$PROCESS_DIR/.state"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ---- Helpers ----

log_info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_phase() { echo -e "\n${CYAN}━━━ $* ━━━${NC}\n"; }

get_state() {
    if [[ -f "$STATE_FILE" ]]; then
        cat "$STATE_FILE"
    else
        echo "uninitialized"
    fi
}

set_state() {
    echo "$1" > "$STATE_FILE"
    log_ok "状态已更新: $1"
}

require_state() {
    local current
    current=$(get_state)
    local required="$1"
    if [[ "$current" != "$required" ]]; then
        log_error "当前状态是 '$current'，但此操作需要状态 '$required'"
        log_info "运行 './process-cli.sh status' 查看当前进度"
        exit 1
    fi
}

require_state_at_least() {
    local current
    current=$(get_state)
    local phases=("uninitialized" "seed" "diverge" "converge" "skeleton" "branching" "stabilize" "postmortem" "done")
    local current_idx=-1
    local required_idx=-1
    for i in "${!phases[@]}"; do
        [[ "${phases[$i]}" == "$current" ]] && current_idx=$i
        [[ "${phases[$i]}" == "$1" ]] && required_idx=$i
    done
    if (( current_idx < required_idx )); then
        log_error "当前状态 '$current' 还未到达 '$1' 阶段"
        exit 1
    fi
}

require_file() {
    if [[ ! -f "$1" ]]; then
        log_error "缺少必要文件: $1"
        exit 1
    fi
}

validate_yaml_fields() {
    local file="$1"
    shift
    local missing=()
    for field in "$@"; do
        if ! grep -q "^${field}:" "$file" 2>/dev/null; then
            missing+=("$field")
        fi
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "文件 $file 缺少必填字段: ${missing[*]}"
        return 1
    fi
    return 0
}

# ---- AI Provider Layer ----
# 支持多供应商: claude, openai, ollama, manual
# 配置文件: .process/.ai_config 或环境变量

AI_CONFIG_FILE="$PROCESS_DIR/.ai_config"
AI_PROVIDER="${AI_PROVIDER:-auto}"  # auto | claude | openai | ollama | manual

# 初始化 AI 配置
init_ai_config() {
    if [[ ! -f "$AI_CONFIG_FILE" ]]; then
        cat > "$AI_CONFIG_FILE" << 'AI_CONFIG'
# AI Provider 配置
# provider: auto | claude | openai | ollama | manual
provider: auto

# Claude API (Anthropic)
# 支持环境变量: ANTHROPIC_API_KEY, ANTHROPIC_MODEL, ANTHROPIC_BASE_URL
claude:
  api_key: ""           # 或设置环境变量 ANTHROPIC_API_KEY
  model: ""             # 默认: claude-sonnet-4-20250514, 或 ANTHROPIC_MODEL
  base_url: ""          # 默认: https://api.anthropic.com, 或 ANTHROPIC_BASE_URL
  max_tokens: 4096

# OpenAI API
# 支持环境变量: OPENAI_API_KEY, OPENAI_MODEL, OPENAI_BASE_URL
openai:
  api_key: ""           # 或设置环境变量 OPENAI_API_KEY
  model: ""             # 默认: gpt-4o, 或 OPENAI_MODEL
  base_url: ""          # 默认: https://api.openai.com, 或 OPENAI_BASE_URL
  max_tokens: 4096

# Ollama (本地)
ollama:
  model: "llama3"
  endpoint: "http://localhost:11434"

# 其他设置
settings:
  auto_save: true        # 自动保存 AI 输出到对应文件
  show_prompt: false     # 是否显示完整 prompt
  timeout: 120           # API 调用超时（秒）
AI_CONFIG
        log_info "已创建 AI 配置文件: $AI_CONFIG_FILE"
    fi
}

# 读取配置值
get_ai_config() {
    local key="$1"
    local default="${2:-}"
    if [[ -f "$AI_CONFIG_FILE" ]]; then
        local value
        # 提取值并去除引号和首尾空格
        value=$(grep "^${key}:" "$AI_CONFIG_FILE" 2>/dev/null | head -1 | sed 's/^[^:]*://' | tr -d '"' | tr -d "'" | xargs || echo "")
        if [[ -n "$value" ]]; then
            echo "$value"
            return
        fi
    fi
    echo "$default"
}

# 读取嵌套配置值 (如 claude.api_key)
# 用法: get_nested_config "section" "key" "default"
get_nested_config() {
    local section="$1"
    local key="$2"
    local default="${3:-}"
    
    if [[ ! -f "$AI_CONFIG_FILE" ]]; then
        echo "$default"
        return
    fi
    
    # 找到 section 后面的 key
    local in_section=false
    local value=""
    
    while IFS= read -r line; do
        # 检查是否进入目标 section
        if [[ "$line" =~ ^${section}: ]]; then
            in_section=true
            continue
        fi
        
        # 检查是否离开 section (遇到新的顶级 key)
        if [[ "$in_section" == "true" && "$line" =~ ^[a-zA-Z] && ! "$line" =~ ^[[:space:]] ]]; then
            break
        fi
        
        # 在 section 内查找 key
        if [[ "$in_section" == "true" && "$line" =~ ^[[:space:]]+${key}: ]]; then
            value=$(echo "$line" | sed 's/^[^:]*://' | tr -d '"' | tr -d "'" | sed 's/#.*//' | xargs)
            break
        fi
    done < "$AI_CONFIG_FILE"
    
    if [[ -n "$value" ]]; then
        echo "$value"
    else
        echo "$default"
    fi
}

# 检测可用的 AI Provider
detect_ai_provider() {
    local configured
    configured=$(get_ai_config "provider" "auto")
    
    if [[ "$configured" != "auto" ]]; then
        echo "$configured"
        return
    fi
    
    # 自动检测顺序: claude > openai > ollama > manual
    
    # 1. 检测 Claude CLI 或 API Key
    if command -v claude &> /dev/null; then
        echo "claude-cli"
        return
    fi
    if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
        echo "claude"
        return
    fi
    
    # 2. 检测 OpenAI
    if [[ -n "${OPENAI_API_KEY:-}" ]]; then
        echo "openai"
        return
    fi
    
    # 3. 检测 Ollama
    if command -v ollama &> /dev/null && curl -s "http://localhost:11434/api/tags" &> /dev/null; then
        echo "ollama"
        return
    fi
    
    # 4. Fallback 到手动模式
    echo "manual"
}

# 清理 AI 输出（去除 markdown 代码块标记等）
clean_ai_output() {
    local content="$1"
    # 使用兼容 macOS 和 Linux 的方式清理
    # 去除开头的 ```yaml 或 ``` 行
    content=$(echo "$content" | grep -v '^```[a-z]*$' | grep -v '^```$')
    # 去除可能的空行开头
    content=$(echo "$content" | awk 'NF{found=1} found')
    echo "$content"
}

# 调用 AI API（带清理和重试）
# 用法: call_ai "prompt" [output_file] [max_retries]
call_ai() {
    local prompt="$1"
    local output_file="${2:-}"
    local max_retries="${3:-2}"
    local provider
    provider=$(detect_ai_provider)
    
    log_info "使用 AI Provider: $provider"
    
    local attempt=1
    local result
    
    while (( attempt <= max_retries )); do
        if (( attempt > 1 )); then
            log_warn "重试第 $attempt 次..."
        fi
        
        case "$provider" in
            claude-cli)
                result=$(call_ai_provider_claude_cli "$prompt")
                ;;
            claude)
                result=$(call_ai_provider_claude_api "$prompt")
                ;;
            openai)
                result=$(call_ai_provider_openai "$prompt")
                ;;
            ollama)
                result=$(call_ai_provider_ollama "$prompt")
                ;;
            manual)
                call_ai_manual "$prompt" "$output_file"
                return $?
                ;;
            *)
                log_error "未知的 AI provider: $provider"
                return 1
                ;;
        esac
        
        # 检查是否成功获取结果
        if [[ -n "$result" ]]; then
            # 清理输出
            result=$(clean_ai_output "$result")
            
            # 保存到文件
            if [[ -n "$output_file" ]]; then
                echo "$result" > "$output_file"
                log_ok "AI 输出已保存到: $output_file"
            else
                echo "$result"
            fi
            return 0
        fi
        
        attempt=$((attempt + 1))
    done
    
    log_error "AI 调用失败（已重试 $max_retries 次）"
    return 1
}

# Claude CLI 调用（返回内容）
call_ai_provider_claude_cli() {
    local prompt="$1"
    local model
    model=$(get_nested_config "claude" "model" "claude-sonnet-4-20250514")
    local max_tokens
    max_tokens=$(get_nested_config "claude" "max_tokens" "8192")
    
    log_info "调用 Claude CLI (model: $model, max_tokens: $max_tokens)..." >&2
    
    local response
    if response=$(echo "$prompt" | claude -m "$model" --max-tokens "$max_tokens" 2>&1); then
        echo "$response"
    else
        log_error "Claude CLI 调用失败: $response" >&2
        return 1
    fi
}

# Claude API 直接调用（返回内容）
# 支持环境变量: ANTHROPIC_API_KEY, ANTHROPIC_MODEL, ANTHROPIC_BASE_URL
call_ai_provider_claude_api() {
    local prompt="$1"
    
    # 优先级: 环境变量 > 配置文件 > 默认值
    local api_key="${ANTHROPIC_API_KEY:-$(get_nested_config "claude" "api_key")}"
    local model="${ANTHROPIC_MODEL:-$(get_nested_config "claude" "model" "claude-sonnet-4-20250514")}"
    local base_url="${ANTHROPIC_BASE_URL:-$(get_nested_config "claude" "base_url" "https://api.anthropic.com")}"
    local max_tokens
    max_tokens=$(get_nested_config "claude" "max_tokens" "8192")
    
    if [[ -z "$api_key" ]]; then
        log_error "未设置 ANTHROPIC_API_KEY" >&2
        return 1
    fi
    
    # 确保 base_url 没有尾部斜杠
    base_url="${base_url%/}"
    
    # 日志输出到 stderr，避免污染返回内容
    log_info "调用 Claude API..." >&2
    log_info "  Base URL: $base_url" >&2
    log_info "  Model: $model" >&2
    log_info "  Max Tokens: $max_tokens" >&2
    
    local escaped_prompt
    escaped_prompt=$(echo "$prompt" | jq -Rs .)
    
    local response
    response=$(curl -s --max-time 180 \
        -H "Content-Type: application/json" \
        -H "x-api-key: $api_key" \
        -H "anthropic-version: 2023-06-01" \
        -d "{
            \"model\": \"$model\",
            \"max_tokens\": $max_tokens,
            \"messages\": [{\"role\": \"user\", \"content\": $escaped_prompt}]
        }" \
        "${base_url}/v1/messages" 2>&1)
    
    # 解析响应
    local content
    content=$(echo "$response" | jq -r '.content[0].text // empty' 2>/dev/null)
    
    if [[ -n "$content" ]]; then
        echo "$content"
    else
        local error
        error=$(echo "$response" | jq -r '.error.message // empty' 2>/dev/null)
        log_error "Claude API 调用失败: ${error:-$response}" >&2
        return 1
    fi
}

# OpenAI API 调用（返回内容）
# 支持环境变量: OPENAI_API_KEY, OPENAI_MODEL, OPENAI_BASE_URL
call_ai_provider_openai() {
    local prompt="$1"
    
    # 优先级: 环境变量 > 配置文件 > 默认值
    local api_key="${OPENAI_API_KEY:-$(get_nested_config "openai" "api_key")}"
    local model="${OPENAI_MODEL:-$(get_nested_config "openai" "model" "gpt-4o")}"
    local base_url="${OPENAI_BASE_URL:-$(get_nested_config "openai" "base_url" "https://api.openai.com")}"
    local max_tokens
    max_tokens=$(get_nested_config "openai" "max_tokens" "8192")
    
    if [[ -z "$api_key" ]]; then
        log_error "未设置 OPENAI_API_KEY" >&2
        return 1
    fi
    
    # 确保 base_url 没有尾部斜杠
    base_url="${base_url%/}"
    
    # 日志输出到 stderr，避免污染返回内容
    log_info "调用 OpenAI API..." >&2
    log_info "  Base URL: $base_url" >&2
    log_info "  Model: $model" >&2
    log_info "  Max Tokens: $max_tokens" >&2
    
    local escaped_prompt
    escaped_prompt=$(echo "$prompt" | jq -Rs .)
    
    local response
    response=$(curl -s --max-time 180 \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $api_key" \
        -d "{
            \"model\": \"$model\",
            \"max_tokens\": $max_tokens,
            \"messages\": [{\"role\": \"user\", \"content\": $escaped_prompt}]
        }" \
        "${base_url}/v1/chat/completions" 2>&1)
    
    local content
    content=$(echo "$response" | jq -r '.choices[0].message.content // empty' 2>/dev/null)
    
    if [[ -n "$content" ]]; then
        echo "$content"
    else
        local error
        error=$(echo "$response" | jq -r '.error.message // empty' 2>/dev/null)
        log_error "OpenAI API 调用失败: ${error:-$response}" >&2
        return 1
    fi
}

# Ollama 本地调用（返回内容）
call_ai_provider_ollama() {
    local prompt="$1"
    local model
    model=$(get_nested_config "ollama" "model" "llama3")
    local endpoint
    endpoint=$(get_nested_config "ollama" "endpoint" "http://localhost:11434")
    
    log_info "调用 Ollama (model: $model)..." >&2
    
    local escaped_prompt
    escaped_prompt=$(echo "$prompt" | jq -Rs .)
    
    local response
    response=$(curl -s --max-time 300 \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$model\",
            \"prompt\": $escaped_prompt,
            \"stream\": false
        }" \
        "${endpoint}/api/generate" 2>&1)
    
    local content
    content=$(echo "$response" | jq -r '.response // empty' 2>/dev/null)
    
    if [[ -n "$content" ]]; then
        echo "$content"
    else
        log_error "Ollama 调用失败: $response" >&2
        return 1
    fi
}

# 手动模式 (打印 prompt，等待用户操作)
call_ai_manual() {
    local prompt="$1"
    local output_file="${2:-}"
    
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  AI 任务 (手动模式)${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
    echo ""
    echo "$prompt"
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
    echo ""
    
    if [[ -n "$output_file" ]]; then
        log_info "请将 AI 输出保存到: $output_file"
        log_info "完成后运行相应的 validate 命令"
    fi
    
    # 尝试复制到剪贴板
    if command -v pbcopy &> /dev/null; then
        echo "$prompt" | pbcopy
        log_ok "Prompt 已复制到剪贴板"
    elif command -v xclip &> /dev/null; then
        echo "$prompt" | xclip -selection clipboard
        log_ok "Prompt 已复制到剪贴板"
    fi
    
    return 0
}

# 设置 AI Provider
cmd_ai_config() {
    local subcmd="${1:-show}"
    
    case "$subcmd" in
        show)
            if [[ -f "$AI_CONFIG_FILE" ]]; then
                log_phase "当前 AI 配置"
                cat "$AI_CONFIG_FILE"
                echo ""
                log_info "检测到的 Provider: $(detect_ai_provider)"
            else
                log_warn "AI 配置文件不存在，运行 init 创建"
            fi
            ;;
        set-provider)
            local provider="${2:?用法: ai-config set-provider <claude|openai|ollama|manual|auto>}"
            if [[ -f "$AI_CONFIG_FILE" ]]; then
                sed -i.bak "s/^provider:.*/provider: $provider/" "$AI_CONFIG_FILE"
                rm -f "${AI_CONFIG_FILE}.bak"
                log_ok "AI Provider 已设置为: $provider"
            else
                log_error "请先运行 init"
            fi
            ;;
        test)
            log_phase "测试 AI 连接"
            local test_prompt="请回复 'Hello, process-cli!' 来确认连接正常。"
            if call_ai "$test_prompt"; then
                log_ok "AI 连接测试成功"
            else
                log_error "AI 连接测试失败"
            fi
            ;;
        *)
            echo "用法: ai-config <show|set-provider|test> [args]"
            ;;
    esac
}

# ---- Commands ----

cmd_init() {
    log_phase "Phase 0: 初始化项目种子"

    mkdir -p "$PROCESS_DIR" "$BRANCHES_DIR"

    if [[ -f "$SEED_FILE" ]]; then
        log_warn "seed.yaml 已存在，跳过生成"
    else
        cat > "$SEED_FILE" << 'SEED_TEMPLATE'
# === Project Seed ===
# 请填写以下所有字段

idea: ""
# 一句话描述核心想法

target_user: ""
# 谁会用这个？具体场景是什么？

constraints:
  - ""
# 硬约束列表

non_goals:
  - ""
# 明确不做的事

success_criteria:
  - ""
# 可验证的成功标准

reversibility_budget: "medium"
# high | medium | low
# high = 可以大胆实验; low = 每步都要可回退
SEED_TEMPLATE
        log_ok "已生成 $SEED_FILE"
        log_info "请编辑 seed.yaml 填写项目信息，然后运行 './process-cli.sh seed-validate'"
    fi

    # Initialize other files
    [[ -f "$REJECTED_FILE" ]] || cat > "$REJECTED_FILE" << 'EOF'
# Rejected Approaches

记录所有被否决的方案及其理由，防止重复踩坑。

---
EOF

    [[ -f "$DECISIONS_FILE" ]] || cat > "$DECISIONS_FILE" << 'EOF'
# Decisions Log
decisions: []
EOF

    [[ -f "$LEARNINGS_FILE" ]] || cat > "$LEARNINGS_FILE" << 'EOF'
# Learnings (实时记录)
learnings: []
EOF

    [[ -f "$FRICTION_FILE" ]] || cat > "$FRICTION_FILE" << 'EOF'
# Friction Points
friction_points: []
EOF

    # 初始化 AI 配置
    init_ai_config

    set_state "seed"
    log_info "项目已初始化。下一步: 编辑 $SEED_FILE 然后运行 seed-validate"
    log_info "AI 配置: $AI_CONFIG_FILE (可选: 配置 API Key)" 
}

cmd_seed_validate() {
    log_phase "验证 Seed"
    require_file "$SEED_FILE"

    local errors=0

    # Check required fields exist and are non-empty
    for field in idea target_user constraints non_goals success_criteria reversibility_budget; do
        if ! grep -q "^${field}:" "$SEED_FILE"; then
            log_error "缺少字段: $field"
            errors=$((errors + 1))
        fi
    done

    # Check for empty string values
    if grep -qE '^idea:\s*""' "$SEED_FILE"; then
        log_error "idea 字段为空"
        errors=$((errors + 1))
    fi

    if grep -qE '^target_user:\s*""' "$SEED_FILE"; then
        log_error "target_user 字段为空"
        errors=$((errors + 1))
    fi

    if [[ $errors -gt 0 ]]; then
        log_error "验证失败，有 $errors 个错误"
        exit 1
    fi

    set_state "seed"
    log_ok "Seed 验证通过"
    log_info "下一步: 运行 './process-cli.sh diverge' 开始多模型发散探索"
}

cmd_diverge() {
    log_phase "Phase 1: Diverge — 多模型发散探索"
    require_state "seed"
    require_file "$SEED_FILE"

    local seed_content
    seed_content=$(cat "$SEED_FILE")

    # 构建 prompt
    local prompt
    prompt=$(cat << PROMPT_END
你是一个软件架构师。请阅读以下项目 seed，然后生成 ≥2 个独立的技术方案。

--- SEED ---
$seed_content
--- END SEED ---

要求：
1. 每个方案必须包含：架构草图（文字描述）、核心取舍、最大风险点
2. 每个方案检查与每条 constraint 的对齐
3. 方案之间要有实质差异（不同架构/技术/取舍方向）
4. 最后生成比较表

请只输出 YAML 格式，不要其他解释：

proposals:
  - name: "方案A"
    summary: "一句话描述"
    architecture: |
      多行架构描述
    tradeoffs:
      - "取舍1"
    risks:
      - "风险1"
    constraint_alignment:
      约束1: "pass | partial | fail"
  - name: "方案B"
    ...
comparison_dimensions:
  - dimension: "维度名"
    ranking: ["A", "B"]
    notes: "说明"
PROMPT_END
)

    log_info "调用 AI 进行发散探索..."
    
    # 调用 AI
    if call_ai "$prompt" "$DIVERGE_FILE"; then
        set_state "diverge"
        log_ok "AI 发散完成"
        log_info "运行 './process-cli.sh diverge-validate' 验证输出"
    else
        set_state "diverge"
        log_warn "AI 调用未完成，请手动创建 $DIVERGE_FILE"
        log_info "完成后运行 './process-cli.sh diverge-validate' 验证"
    fi
}

cmd_diverge_validate() {
    log_phase "验证 Diverge 输出"
    require_file "$DIVERGE_FILE"

    if ! grep -q "proposals:" "$DIVERGE_FILE"; then
        log_error "diverge_summary.yaml 缺少 proposals 字段"
        exit 1
    fi

    local proposal_count
    proposal_count=$(grep -c "^  - name:" "$DIVERGE_FILE" || echo "0")
    if (( proposal_count < 2 )); then
        log_error "至少需要 2 个方案，当前只有 $proposal_count 个"
        exit 1
    fi

    if ! grep -q "comparison_dimensions:" "$DIVERGE_FILE"; then
        log_error "缺少 comparison_dimensions 比较表"
        exit 1
    fi

    set_state "diverge"
    log_ok "Diverge 输出验证通过 ($proposal_count 个方案)"
    log_info "下一步: 运行 './process-cli.sh converge' 开始剪枝和规则提取"
}

cmd_converge() {
    log_phase "Phase 2: Converge — 剪枝 + 规则提取"
    require_state "diverge"
    require_file "$DIVERGE_FILE"

    local diverge_content
    diverge_content=$(cat "$DIVERGE_FILE")
    local seed_content
    seed_content=$(cat "$SEED_FILE")

    local prompt
    prompt=$(cat << PROMPT_END
你是一个软件架构师。请阅读 seed 和发散方案，完成以下任务：

--- SEED ---
$seed_content
--- END SEED ---

--- DIVERGE SUMMARY ---
$diverge_content
--- END DIVERGE ---

任务：
1. 裁决：选择/混合方案，淘汰的方案要列出原因
2. 从选中方案中提取规则

请只输出 YAML 格式，不要其他解释：

invariants:
  - id: "INV-001"
    rule: "规则描述"
    rationale: "为什么"
    added_in_phase: 2
    frozen: false

conventions:
  - id: "CONV-001"
    rule: "约定描述"
    rationale: "为什么"

conflict_resolution:
  policy: "human_final_say"

rejected_approaches:
  - name: "被淘汰的方案名"
    reason: "淘汰原因"

selected_approach:
  name: "选中的方案名"
  rationale: "选择原因"
PROMPT_END
)

    log_info "调用 AI 进行方案收敛..."
    
    if call_ai "$prompt" "$RULES_FILE"; then
        set_state "converge"
        log_ok "AI 收敛完成"
        log_info "运行 './process-cli.sh converge-validate' 验证输出"
    else
        set_state "converge"
        log_warn "AI 调用未完成，请手动创建 $RULES_FILE"
        log_info "完成后运行 './process-cli.sh converge-validate' 验证"
    fi
}

cmd_converge_validate() {
    log_phase "验证 Converge 输出"
    require_file "$RULES_FILE"

    if ! grep -q "invariants:" "$RULES_FILE"; then
        log_error "rules.yaml 缺少 invariants"
        exit 1
    fi

    local inv_count
    inv_count=$(grep -c "id: \"INV-" "$RULES_FILE" || echo "0")
    if (( inv_count < 1 )); then
        log_error "至少需要 1 条 invariant"
        exit 1
    fi

    if ! grep -q "conflict_resolution:" "$RULES_FILE"; then
        log_error "rules.yaml 缺少 conflict_resolution 策略"
        exit 1
    fi

    set_state "converge"
    log_ok "Converge 验证通过 ($inv_count 条 invariants)"
    log_info "下一步: 运行 './process-cli.sh skeleton' 生成项目骨架"
}

cmd_skeleton() {
    log_phase "Phase 3: Skeleton — 生成项目骨架"
    require_state "converge"
    require_file "$RULES_FILE"
    require_file "$SEED_FILE"

    local rules_content
    rules_content=$(cat "$RULES_FILE")
    local seed_content
    seed_content=$(cat "$SEED_FILE")

    local prompt
    prompt=$(cat << PROMPT_END
你是一个软件架构师。基于以下 seed 和 rules，生成项目骨架配置。

--- SEED ---
$seed_content
--- END SEED ---

--- RULES ---
$rules_content
--- END RULES ---

请只输出 YAML 格式，不要其他解释：

directory_structure:
  - path: "src/core/"
    purpose: "核心业务逻辑"
  - path: "src/commands/"
    purpose: "命令实现"

interfaces: |
  # 核心函数签名（只有签名，不实现）
  # function_name() { :; }  # 描述

rollback_template: |
  ## 回滚步骤
  1. git revert <commit-range>
  2. 验证: <具体验证命令>
  3. 通知: <谁需要知道>

verification_checklist:
  - check: "所有 invariant 仍然成立"
    automated: true
    command: "make check-invariants"
  - check: "无新的循环依赖"
    automated: false
    command: ""
PROMPT_END
)

    log_info "调用 AI 生成项目骨架..."
    
    if call_ai "$prompt" "$SKELETON_FILE"; then
        set_state "skeleton"
        log_ok "AI 骨架生成完成"
        log_info "运行 './process-cli.sh skeleton-validate' 验证输出"
        log_info "然后执行: git add . && git commit -m 'skeleton' && git tag skeleton-v1"
    else
        set_state "skeleton"
        log_warn "AI 调用未完成，请手动创建 $SKELETON_FILE"
        log_info "完成后运行 './process-cli.sh skeleton-validate' 验证"
    fi
}

cmd_skeleton_validate() {
    log_phase "验证 Skeleton"
    require_file "$SKELETON_FILE"

    for field in directory_structure interfaces rollback_template verification_checklist; do
        if ! grep -q "${field}:" "$SKELETON_FILE"; then
            log_error "skeleton.yaml 缺少字段: $field"
            exit 1
        fi
    done

    # Check git tag
    if git tag -l | grep -q "^skeleton-v1$" 2>/dev/null; then
        log_ok "git tag skeleton-v1 已存在"
    else
        log_warn "git tag skeleton-v1 不存在，请创建"
    fi

    set_state "skeleton"
    log_ok "Skeleton 验证通过"
    log_info "下一步: 运行 './process-cli.sh branch-new <name>' 开始迭代分支"
}

cmd_branch_new() {
    log_phase "Phase 4.1: 新建分支假设"
    require_state_at_least "skeleton"

    local branch_name="${1:?用法: branch-new <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"

    if [[ -f "$branch_file" ]]; then
        log_error "分支 $branch_name 已存在"
        exit 1
    fi

    cat > "$branch_file" << BRANCH_TEMPLATE
# Branch Hypothesis: $branch_name
hypothesis: ""
# 添加 X 功能将使 Y 成为可能

scope:
  files_to_touch: []
  files_not_to_touch: []

invariants_at_risk:
  - ""
# 哪些 invariant 可能受影响

rollback_plan: |
  git revert 到分支起点
  验证: <具体命令>
  影响范围: <哪些模块>

dependencies:
  blocked_by: []
  blocks: []

priority: 5
# 1-10, 数字越小越先做

estimated_complexity: "medium"
# small | medium | large

status: "defined"
# defined → implementing → reviewing → abuse-testing → merged | rejected
BRANCH_TEMPLATE

    set_state "branching"
    log_ok "已创建分支定义: $branch_file"
    log_info "请编辑假设定义，然后运行 './process-cli.sh branch-start $branch_name'"
}

cmd_branch_start() {
    log_phase "Phase 4.2: 开始分支实现"
    local branch_name="${1:?用法: branch-start <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"
    require_file "$branch_file"

    # Validate hypothesis is filled
    if grep -qE '^hypothesis:\s*""' "$branch_file"; then
        log_error "请先填写 hypothesis"
        exit 1
    fi

    # Update status
    if command -v sed &>/dev/null; then
        sed -i.bak 's/^status: "defined"/status: "implementing"/' "$branch_file"
        rm -f "${branch_file}.bak"
    fi

    # Create git branch
    if command -v git &>/dev/null && git rev-parse --is-inside-work-tree &>/dev/null 2>&1; then
        git checkout -b "feature/$branch_name" 2>/dev/null || log_warn "Git 分支创建失败（可能已存在）"
    fi

    log_ok "分支 $branch_name 已进入实现阶段"
    echo ""
    log_info "实现过程中请："
    log_info "  1. 提交消息引用 invariant ID: feat: xxx [INV-001 verified]"
    log_info "  2. 实时记录教训: './process-cli.sh learn \"教训内容\"'"
    log_info "  3. 完成后运行: './process-cli.sh branch-review $branch_name'"
}

cmd_learn() {
    local lesson="${1:?用法: learn \"教训内容\"}"
    local category="${2:-general}"

    cat >> "$LEARNINGS_FILE" << LEARNING
  - timestamp: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    category: "$category"
    lesson: "$lesson"
    phase: "$(get_state)"
LEARNING

    log_ok "教训已记录"
}

cmd_branch_review() {
    log_phase "Phase 4.3: 多角色审查"
    local branch_name="${1:?用法: branch-review <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"
    local review_file="$BRANCHES_DIR/${branch_name}-review.yaml"
    require_file "$branch_file"

    local rules_content
    rules_content=$(cat "$RULES_FILE")
    local branch_content
    branch_content=$(cat "$branch_file")

    # Update status
    sed -i.bak 's/^status: "implementing"/status: "reviewing"/' "$branch_file"
    rm -f "${branch_file}.bak"

    local prompt
    prompt=$(cat << PROMPT_END
你是一个代码审查专家。请以 4 个角色分别审查本分支。

--- RULES ---
$rules_content
--- END RULES ---

--- BRANCH DEF ---
$branch_content
--- END BRANCH DEF ---

角色 1: 安全审计员 - 关注注入、越权、数据泄露
角色 2: 性能工程师 - 关注热路径、延迟、并发
角色 3: 用户代言人 - 用户体验变好还是变差？
角色 4: 维护者 - 6个月后能看懂吗？

请只输出 YAML 格式：

reviews:
  - role: "安全审计员"
    verdict: "pass | conditional_pass | fail"
    issues:
      - severity: "high | medium | low"
        description: "问题描述"
        suggestion: "建议"
  - role: "性能工程师"
    verdict: "..."
    issues: []

conflicts: []  # 角色间冲突，需要 NEEDS_HUMAN_DECISION

overall_verdict: "pass | conditional_pass | fail"
PROMPT_END
)

    log_info "调用 AI 进行多角色审查..."
    
    if call_ai "$prompt" "$review_file"; then
        log_ok "AI 审查完成"
        log_info "审查结果已保存到: $review_file"
        log_info "下一步: './process-cli.sh branch-abuse $branch_name'"
    else
        log_warn "AI 调用未完成，请手动创建 $review_file"
    fi
}

cmd_branch_abuse() {
    log_phase "Phase 4.4: 滥用测试"
    local branch_name="${1:?用法: branch-abuse <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"
    local abuse_file="$BRANCHES_DIR/${branch_name}-abuse.yaml"
    require_file "$branch_file"

    sed -i.bak 's/^status: "reviewing"/status: "abuse-testing"/' "$branch_file"
    rm -f "${branch_file}.bak"

    local branch_content
    branch_content=$(cat "$branch_file")

    local prompt
    prompt=$(cat << PROMPT_END
你是一个安全测试专家。以"恶意用户"视角测试本分支的代码。

--- BRANCH DEF ---
$branch_content
--- END BRANCH DEF ---

测试类别：
1. 边界输入 — 空值、超长、特殊字符、类型错误
2. 违反预期的操作序列 — 乱序调用、重复调用、并发调用
3. 资源耗尽 — 大量数据、频繁请求、内存填充

请只输出 YAML 格式：

abuse_tests:
  - category: "边界输入"
    test: "测试描述"
    result: "pass | fail"
    severity: "high | medium | low"
    fix_suggestion: "修复建议"
  - category: "操作序列"
    test: "..."
    result: "..."
    severity: "..."
    fix_suggestion: "..."

summary:
  total_tests: N
  passed: N
  failed: N
  high_severity_issues: N
PROMPT_END
)

    log_info "调用 AI 进行滥用测试..."
    
    if call_ai "$prompt" "$abuse_file"; then
        log_ok "AI 滥用测试完成"
        log_info "测试结果已保存到: $abuse_file"
        log_info "下一步: './process-cli.sh branch-gate $branch_name'"
    else
        log_warn "AI 调用未完成，请手动创建 $abuse_file"
    fi
}

cmd_branch_gate() {
    log_phase "Phase 4.5: 合并门检查"
    local branch_name="${1:?用法: branch-gate <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"
    local abuse_file="$BRANCHES_DIR/${branch_name}-abuse.yaml"
    require_file "$branch_file"

    local pass=true

    echo "合并检查清单:"
    echo ""

    # Check reviews exist (simplified - in real usage, check review results)
    if grep -q 'status: "abuse-testing"' "$branch_file" || grep -q 'status: "reviewing"' "$branch_file"; then
        echo -e "  ${GREEN}✓${NC} 已通过审查流程"
    else
        echo -e "  ${YELLOW}?${NC} 审查状态未确认"
    fi

    # Check abuse results
    if [[ -f "$abuse_file" ]]; then
        local high_issues
        high_issues=$(grep -c 'severity: "high"' "$abuse_file" 2>/dev/null || echo "0")
        local high_fails
        high_fails=$(grep -B1 'severity: "high"' "$abuse_file" 2>/dev/null | grep -c 'result: "fail"' || echo "0")
        if (( high_fails > 0 )); then
            echo -e "  ${RED}✗${NC} 有 $high_fails 个 high severity 滥用测试失败"
            pass=false
        else
            echo -e "  ${GREEN}✓${NC} 滥用测试通过 (无 high severity 失败)"
        fi
    else
        echo -e "  ${RED}✗${NC} 缺少滥用测试结果"
        pass=false
    fi

    # Check scope creep
    local not_touch_list
    not_touch_list=$(grep -A100 "files_not_to_touch:" "$branch_file" | grep "^    - " | sed 's/^    - //' | head -20)
    if [[ -n "$not_touch_list" ]]; then
        local scope_violation=false
        while IFS= read -r file; do
            file=$(echo "$file" | tr -d '"' | tr -d "'")
            if [[ -n "$file" ]] && git diff --name-only HEAD~1 2>/dev/null | grep -q "$file"; then
                echo -e "  ${RED}✗${NC} 范围溢出: 修改了 $file"
                scope_violation=true
            fi
        done <<< "$not_touch_list"
        if [[ "$scope_violation" == "false" ]]; then
            echo -e "  ${GREEN}✓${NC} 未超出范围"
        else
            pass=false
        fi
    fi

    # Check tests
    echo -e "  ${YELLOW}?${NC} 请手动确认: 测试是否全部通过?"
    echo -e "  ${YELLOW}?${NC} 请手动确认: invariants 是否全部验证?"
    echo -e "  ${YELLOW}?${NC} 请手动确认: 回滚步骤是否已测试?"

    echo ""
    if [[ "$pass" == "true" ]]; then
        log_ok "自动检查通过。请确认手动检查项后运行 './process-cli.sh branch-merge $branch_name'"
    else
        log_error "存在未通过的检查项，请修复后重新运行"
    fi
}

cmd_branch_merge() {
    log_phase "Phase 4.5: 合并分支"
    local branch_name="${1:?用法: branch-merge <branch-name>}"
    local branch_file="$BRANCHES_DIR/${branch_name}.yaml"
    require_file "$branch_file"

    sed -i.bak 's/^status: ".*"/status: "merged"/' "$branch_file"
    rm -f "${branch_file}.bak"

    log_ok "分支 $branch_name 标记为已合并"
    log_info "请记录使用反馈: './process-cli.sh friction $branch_name \"摩擦点描述\"'"
    log_info "或继续: './process-cli.sh branch-new <next-branch>'"
}

cmd_friction() {
    local branch_name="${1:?用法: friction <branch-name> \"描述\" [severity]}"
    local description="${2:?缺少描述}"
    local severity="${3:-medium}"

    cat >> "$FRICTION_FILE" << FRICTION
  - branch: "$branch_name"
    timestamp: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    description: "$description"
    severity: "$severity"
    action: "pending"
FRICTION

    log_ok "摩擦点已记录 (severity: $severity)"
}

cmd_stabilize() {
    log_phase "Phase 5: 进入稳定化阶段"
    require_state_at_least "branching"

    # Check for unmerged branches
    local unmerged=0
    for f in "$BRANCHES_DIR"/*.yaml; do
        [[ "$f" == *"-abuse.yaml" ]] && continue
        [[ ! -f "$f" ]] && continue
        if ! grep -q 'status: "merged"' "$f" && ! grep -q 'status: "rejected"' "$f"; then
            local name
            name=$(basename "$f" .yaml)
            log_warn "分支 $name 尚未合并"
            unmerged=$((unmerged + 1))
        fi
    done

    if (( unmerged > 0 )); then
        log_warn "有 $unmerged 个分支尚未合并/拒绝，确认要继续稳定化吗？(y/n)"
        read -r confirm
        [[ "$confirm" != "y" ]] && exit 0
    fi

    # Freeze all invariants
    if [[ -f "$RULES_FILE" ]]; then
        sed -i.bak 's/frozen: false/frozen: true/g' "$RULES_FILE"
        rm -f "${RULES_FILE}.bak"
        log_ok "所有 invariants 已冻结"
    fi

    # Check high severity friction
    if [[ -f "$FRICTION_FILE" ]]; then
        local high_friction
        high_friction=$(grep -c 'severity: "high"' "$FRICTION_FILE" 2>/dev/null || echo "0")
        if (( high_friction > 0 )); then
            log_warn "有 $high_friction 个 high severity 摩擦点需要在稳定化阶段解决"
        fi
    fi

    set_state "stabilize"
    log_ok "已进入稳定化阶段"
    echo ""
    echo "稳定化规则:"
    echo "  - 不允许添加新 invariant"
    echo "  - 只允许 bugfix 分支"
    echo "  - high severity 摩擦点必须解决"
    echo "  - 用户痛感 > 代码优雅"
    echo ""
    log_info "修完所有必要 bug 后，运行 './process-cli.sh postmortem'"
}

cmd_postmortem() {
    log_phase "Phase 6: Postmortem"
    require_state "stabilize"

    # Gather all process artifacts
    log_info "收集所有过程产物用于回顾..."

    local learnings_content=""
    local friction_content=""
    local rules_content=""
    
    [[ -f "$LEARNINGS_FILE" ]] && learnings_content=$(cat "$LEARNINGS_FILE")
    [[ -f "$FRICTION_FILE" ]] && friction_content=$(cat "$FRICTION_FILE")
    [[ -f "$RULES_FILE" ]] && rules_content=$(cat "$RULES_FILE")

    local prompt
    prompt=$(cat << PROMPT_END
你是一个项目复盘专家。请回顾本项目的所有过程产物，生成复盘报告。

--- LEARNINGS ---
$learnings_content
--- END LEARNINGS ---

--- FRICTION ---
$friction_content
--- END FRICTION ---

--- RULES ---
$rules_content
--- END RULES ---

请只输出 YAML 格式：

rules_that_should_exist_earlier:
  - rule: "规则描述"
    current_phase_added: 2
    ideal_phase: 1
    reason: "为什么应该更早添加"

rejected_approaches_review:
  - approach: "方案名"
    original_rejection_reason: "原因"
    retrospective_verdict: "rejection_correct | should_reconsider"
    note: "补充说明"

process_improvements:
  - description: "改进描述"
    priority: "high | medium | low"
    action: "update_process_spec | add_template | improve_tooling"

learnings_summary:
  - category: "技术 | 流程 | 协作"
    lesson: "教训"
    actionable: "可行动的建议"

metrics:
  total_branches: N
  merged_branches: N
  rejected_branches: N
  friction_points: N
  high_severity_issues: N
PROMPT_END
)

    log_info "调用 AI 生成复盘报告..."
    
    if call_ai "$prompt" "$POSTMORTEM_FILE"; then
        set_state "postmortem"
        log_ok "AI 复盘完成"
        log_info "复盘报告已保存到: $POSTMORTEM_FILE"
        log_info "下一步: './process-cli.sh done'"
    else
        set_state "postmortem"
        log_warn "AI 调用未完成，请手动创建 $POSTMORTEM_FILE"
        log_info "完成后运行 './process-cli.sh done'"
    fi
}

cmd_done() {
    set_state "done"
    log_ok "流程完成！"
    echo ""
    cmd_status
}

cmd_status() {
    local state
    state=$(get_state)

    echo ""
    echo -e "${CYAN}═══ 项目流程状态 ═══${NC}"
    echo ""

    local phases=("seed:Phase 0 - Seed" "diverge:Phase 1 - Diverge" "converge:Phase 2 - Converge" "skeleton:Phase 3 - Skeleton" "branching:Phase 4 - Branch Loop" "stabilize:Phase 5 - Stabilize" "postmortem:Phase 6 - Postmortem" "done:Complete")
    local phases_keys=("seed" "diverge" "converge" "skeleton" "branching" "stabilize" "postmortem" "done")

    local current_idx=-1
    for i in "${!phases_keys[@]}"; do
        [[ "${phases_keys[$i]}" == "$state" ]] && current_idx=$i
    done

    for i in "${!phases[@]}"; do
        IFS=':' read -r key label <<< "${phases[$i]}"
        if [[ "$key" == "$state" ]]; then
            echo -e "  ${GREEN}→ $label ${NC}  ← 当前"
        elif (( i < current_idx )); then
            echo -e "  ${GREEN}✓ $label${NC}"
        else
            echo -e "  ${BLUE}○ $label${NC}"
        fi
    done

    echo ""

    # Show branch status if in branching phase
    if [[ "$state" == "branching" || "$state" == "stabilize" ]]; then
        echo -e "${CYAN}分支状态:${NC}"
        for f in "$BRANCHES_DIR"/*.yaml; do
            [[ "$f" == *"-abuse.yaml" ]] && continue
            [[ ! -f "$f" ]] && continue
            local name
            name=$(basename "$f" .yaml)
            local status
            status=$(grep "^status:" "$f" | head -1 | sed 's/status: "//' | sed 's/"//')
            case "$status" in
                merged)       echo -e "  ${GREEN}✓ $name${NC} ($status)" ;;
                rejected)     echo -e "  ${RED}✗ $name${NC} ($status)" ;;
                implementing) echo -e "  ${YELLOW}▶ $name${NC} ($status)" ;;
                *)            echo -e "  ${BLUE}○ $name${NC} ($status)" ;;
            esac
        done
        echo ""
    fi

    # Show file status
    echo -e "${CYAN}产物文件:${NC}"
    for f in "$SEED_FILE" "$DIVERGE_FILE" "$RULES_FILE" "$SKELETON_FILE" "$FRICTION_FILE" "$LEARNINGS_FILE" "$POSTMORTEM_FILE" "$REJECTED_FILE"; do
        if [[ -f "$f" ]]; then
            echo -e "  ${GREEN}✓${NC} $f"
        else
            echo -e "  ${BLUE}○${NC} $f"
        fi
    done
    echo ""
}

cmd_help() {
    cat << 'HELP'

╔══════════════════════════════════════════════════════════╗
║           process-cli.sh — 开发流程引擎                  ║
╚══════════════════════════════════════════════════════════╝

初始化:
  init                    初始化项目和 .process/ 目录
  seed-validate           验证 seed.yaml

发散探索:
  diverge                 开始多模型发散 (Phase 1)
  diverge-validate        验证发散输出

收敛决策:
  converge                开始剪枝和规则提取 (Phase 2)
  converge-validate       验证收敛输出

项目骨架:
  skeleton                生成项目骨架 (Phase 3)
  skeleton-validate       验证骨架

迭代分支 (Phase 4):
  branch-new <name>       创建新分支假设
  branch-start <name>     开始实现
  branch-review <name>    触发多角色审查
  branch-abuse <name>     触发滥用测试
  branch-gate <name>      合并门检查
  branch-merge <name>     标记合并

实用工具:
  learn "内容" [类别]      记录教训 (实时)
  friction <branch> "描述" [severity]  记录摩擦点

稳定化与收尾:
  stabilize               进入稳定化 (Phase 5)
  postmortem              执行 Postmortem (Phase 6)
  done                    标记完成

状态:
  status                  查看当前进度
  help                    显示此帮助

AI 配置:
  ai-config show          查看当前 AI 配置
  ai-config set-provider <provider>  设置 AI 提供商
  ai-config test          测试 AI 连接

支持的 AI Provider:
  auto     - 自动检测 (Claude CLI → Claude API → OpenAI → Ollama → 手动)
  claude   - Anthropic Claude API (需要 ANTHROPIC_API_KEY)
  openai   - OpenAI API (需要 OPENAI_API_KEY)
  ollama   - 本地 Ollama
  manual   - 手动模式 (打印 prompt 复制到剪贴板)

HELP
}

# ---- Main ----

cmd="${1:-help}"
shift 2>/dev/null || true

case "$cmd" in
    init)               cmd_init "$@" ;;
    seed-validate)      cmd_seed_validate "$@" ;;
    diverge)            cmd_diverge "$@" ;;
    diverge-validate)   cmd_diverge_validate "$@" ;;
    converge)           cmd_converge "$@" ;;
    converge-validate)  cmd_converge_validate "$@" ;;
    skeleton)           cmd_skeleton "$@" ;;
    skeleton-validate)  cmd_skeleton_validate "$@" ;;
    branch-new)         cmd_branch_new "$@" ;;
    branch-start)       cmd_branch_start "$@" ;;
    branch-review)      cmd_branch_review "$@" ;;
    branch-abuse)       cmd_branch_abuse "$@" ;;
    branch-gate)        cmd_branch_gate "$@" ;;
    branch-merge)       cmd_branch_merge "$@" ;;
    learn)              cmd_learn "$@" ;;
    friction)           cmd_friction "$@" ;;
    stabilize)          cmd_stabilize "$@" ;;
    postmortem)         cmd_postmortem "$@" ;;
    done)               cmd_done "$@" ;;
    status)             cmd_status "$@" ;;
    ai-config)          cmd_ai_config "$@" ;;
    help|--help|-h)     cmd_help ;;
    *)                  log_error "未知命令: $cmd"; cmd_help; exit 1 ;;
esac
