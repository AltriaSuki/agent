# Development Process Specification v2

## Overview

```
Phase 0: Seed        → 结构化输入
Phase 1: Diverge     → 多模型发散探索
Phase 2: Converge    → 剪枝 + 规则提取
Phase 3: Skeleton    → 骨架生成 + 不变量
Phase 4: Branch Loop → 假设驱动迭代
Phase 5: Stabilize   → 冻结 + 仅修缺陷
Phase 6: Postmortem  → 回顾 + 规则回流
```

---

## Phase 0: Seed (Human → Structured Input)

Human 提供以下结构化输入，缺一不可：

```yaml
# .process/seed.yaml
idea: "一句话描述核心想法"
target_user: "谁会用这个？具体场景是什么？"
constraints:
  - "硬约束1 (e.g. 必须纯离线运行)"
  - "硬约束2"
non_goals:
  - "明确不做的事1"
  - "明确不做的事2"
success_criteria:
  - "可验证的成功标准1"
  - "可验证的成功标准2"
reversibility_budget: "high | medium | low"
# high = 可以大胆实验; low = 每步都要可回退
```

**退出标准**: seed.yaml 存在且所有字段非空。

---

## Phase 1: Diverge (Multi-Model Brainstorm)

### 输入
seed.yaml

### 过程
1. 用 ≥2 个不同模型/角色分别独立生成方案（不互相看）
2. 每个方案必须包含：
   - 架构草图（文字描述即可）
   - 核心取舍说明（选了什么，放弃了什么）
   - 最大风险点
   - 与 seed 中每条 constraint 的对齐检查
3. 汇总为结构化比较表

### 输出
```yaml
# .process/diverge_summary.yaml
proposals:
  - name: "方案A"
    summary: "..."
    tradeoffs: ["..."]
    risks: ["..."]
    constraint_alignment:
      constraint_1: "pass | partial | fail"
  - name: "方案B"
    # ...
comparison_dimensions:
  - dimension: "复杂度"
    ranking: ["A", "B"]
  - dimension: "可逆性"
    ranking: ["B", "A"]
```

**退出标准**: ≥2 个方案生成，比较表填完，无遗漏维度。

---

## Phase 2: Converge (Prune + Rule Extraction)

### 输入
seed.yaml + diverge_summary.yaml

### 过程
1. Human + AI 共同裁决：
   - 淘汰方案需注明理由 → 写入 REJECTED_APPROACHES.md
   - 选中方案可以是混合方案
2. 从选中方案中提取显式规则：

```yaml
# .process/rules.yaml
invariants:
  - id: "INV-001"
    rule: "所有 API 响应必须在 200ms 内返回"
    rationale: "用户体验硬约束"
    added_in_phase: 2
    frozen: false

conventions:
  - id: "CONV-001"
    rule: "文件命名用 kebab-case"
    rationale: "团队一致性"

conflict_resolution:
  policy: "human_final_say"
  # 当 AI 角色间意见冲突时:
  # 1. 列出各方论点
  # 2. 用 invariants 做裁判
  # 3. invariants 也无法裁决时，human 决定
  # 4. 决定连同理由记录在 decisions_log.yaml
```

3. 更新 REJECTED_APPROACHES.md

### 输出
- rules.yaml
- REJECTED_APPROACHES.md (追加)
- decisions_log.yaml (首次创建)

**退出标准**: rules.yaml 至少有 1 条 invariant，REJECTED_APPROACHES 中每条有理由。

---

## Phase 3: Skeleton

### 输入
seed.yaml + rules.yaml

### 过程
AI 生成：
1. 项目目录结构
2. 核心接口/类型定义（只有签名，没有实现）
3. 不变量检查清单（可自动化验证的尽量自动化）
4. 回滚计划模板

```yaml
# .process/skeleton.yaml
directory_structure:
  - path: "src/core/"
    purpose: "核心业务逻辑"
  - path: "src/adapters/"
    purpose: "外部依赖适配层"

interfaces: |
  // 只定义签名，不实现
  type Processor = (input: RawData) -> Result<Output, Error>

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
    automated: true
    command: "madge --circular src/"
```

### 输出
- 项目骨架代码（已提交到 git）
- skeleton.yaml
- 第一个 git tag: `skeleton-v1`

**退出标准**: 骨架代码可编译/可加载（不需要通过测试），skeleton.yaml 完整。

---

## Phase 4: Branch Loop (Iterative)

每个功能分支遵循以下子流程：

### 4.1 Hypothesis Definition

```yaml
# .process/branches/<branch-name>.yaml
hypothesis: "添加 X 功能将使 Y 成为可能"
scope:
  files_to_touch: ["src/core/processor.ts", "src/adapters/db.ts"]
  files_not_to_touch: ["src/core/auth.ts"]
invariants_at_risk:
  - "INV-001: 可能影响响应时间"
rollback_plan: |
  git revert 到分支起点
  验证: npm test
  影响范围: 仅 processor 模块
dependencies:
  blocked_by: []  # 其他分支名
  blocks: []
priority: 1  # 数字越小越先做
estimated_complexity: "small | medium | large"
```

### 4.2 Implementation

- 正常编码
- 每次提交消息引用相关 invariant ID（如 `feat: add caching [INV-001 verified]`）
- 运行中发现的教训实时记录到 `.process/learnings.yaml`（不等 postmortem）

### 4.3 Multi-Role Review

AI 分别以以下角色审查代码：

| 角色 | 关注点 |
|------|--------|
| 安全审计员 | 注入、越权、数据泄露 |
| 性能工程师 | 热路径、内存、延迟 |
| 用户代言人 | 这个改动让用户体验变好还是变差？ |
| 维护者 | 6个月后能看懂吗？改起来容易吗？ |

每个角色输出：
```yaml
role: "安全审计员"
verdict: "pass | conditional_pass | fail"
issues:
  - severity: "high | medium | low"
    description: "..."
    suggestion: "..."
```

**冲突解决**: 按 rules.yaml 中定义的 conflict_resolution.policy 执行。

### 4.4 Abuse Testing

在合并前，AI 以"恶意用户"视角尝试：
- 边界输入
- 违反预期的操作序列
- 资源耗尽场景

输出记录到 `.process/branches/<branch-name>-abuse.yaml`

### 4.5 Merge Gate

```yaml
# 合并条件 (全部满足才能合并)
merge_criteria:
  - all_reviews_pass: true        # 4.3 全部 pass 或 conditional_pass
  - abuse_issues_resolved: true   # 4.4 无 high severity 未解决
  - invariants_verified: true     # make check-invariants 通过
  - tests_pass: true              # 测试全过
  - no_scope_creep: true          # 没有改 files_not_to_touch 中的文件
  - rollback_tested: true         # 回滚步骤已验证可执行
```

### 4.6 Usage Feedback

- AI 提供使用示例
- Human 实际使用并记录摩擦点到 `.process/friction.yaml`

```yaml
# .process/friction.yaml
- branch: "add-caching"
  friction_points:
    - description: "缓存失效时没有明确提示"
      severity: "medium"
      action: "create_branch | defer | wontfix"
```

### 分支排序规则
1. 按 priority 数值升序
2. 同优先级按依赖拓扑排序
3. 无依赖的可并行

---

## Phase 5: Stabilize

### 触发条件 (满足任一)
- 所有计划分支已合并
- success_criteria（来自 seed.yaml）全部满足
- Human 显式宣布进入稳定化

### 规则
- rules.yaml 中所有 invariant 标记为 `frozen: true`
- 不允许添加新 invariant
- 只允许 bugfix 分支（仍走 Phase 4 流程但简化：跳过 4.1 的 hypothesis，直接写 bug 描述）
- friction.yaml 中 severity=high 的必须在此阶段解决
- **"用户痛感 > 代码优雅"** — 如果用户说某处难用，修它，即使代码因此变丑

### 退出标准
- 无 high severity friction
- 无 failing tests
- invariants 全部通过
- Human 确认可以发布

---

## Phase 6: Postmortem

### 输入
整个 .process/ 目录

### 输出

```yaml
# .process/postmortem.yaml
rules_that_should_exist_earlier:
  - rule: "应该在 Phase 2 就定义 API 版本策略"
    current_phase_added: 4
    ideal_phase: 2

rejected_approaches_review:
  - approach: "方案B-微服务架构"
    original_rejection_reason: "过于复杂"
    retrospective_verdict: "rejection_correct | should_reconsider"

process_improvements:
  - description: "Phase 1 应该限制每个方案的篇幅"
    action: "update_process_spec"

learnings_summary:
  - category: "技术"
    lesson: "SQLite 在并发写入场景下不够用"
  - category: "流程"
    lesson: "abuse testing 在 Phase 4.4 太晚，应该在 4.2 期间持续做"
```

### 回流
- postmortem 中的 process_improvements 反馈到本文档的下一个版本
- rules_that_should_exist_earlier 更新到 rules.yaml 模板中作为提醒

---

## Context Persistence (跨阶段上下文传递)

所有过程产物集中在 `.process/` 目录：

```
.process/
├── seed.yaml                    # Phase 0
├── diverge_summary.yaml         # Phase 1
├── rules.yaml                   # Phase 2+
├── decisions_log.yaml           # Phase 2+
├── skeleton.yaml                # Phase 3
├── branches/
│   ├── <branch-name>.yaml       # Phase 4.1
│   └── <branch-name>-abuse.yaml # Phase 4.4
├── friction.yaml                # Phase 4.6
├── learnings.yaml               # Phase 4.2+ (实时)
├── postmortem.yaml              # Phase 6
└── REJECTED_APPROACHES.md       # Phase 2+
```

每个阶段的 AI 在开始前必须读取前序阶段的所有产物。
