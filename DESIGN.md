# Process CLI — 架构愿景

> **最后更新**: 2026-02-09
> **状态**: Brainstorm 沉淀，待实现

---

## 一、重新定位

Process CLI 不是一个开发流程工具，而是一个**通用决策流水线引擎**。

软件开发是它的第一个"领域包"，但同样的发散-收敛-迭代-复盘模型可以应用于小说创作、建筑设计、课程设计等任何创造性工作。

### 核心理念（不变）

> 在 AI 能瞬间生成一切的时代，人很容易变成橡皮图章。
> Process CLI 不是帮你更快写代码的工具，而是帮你成为更好的决策者。

### 新增理念

> 过程本身就有意义。就像围棋 AI 超过人类后，人类还是在下棋——思考本身是愉悦的。
> Process CLI 的目标不只是产出好的决策，还有让决策过程本身有趣。

---

## 二、核心架构：Pass Engine

### 从 Phase 到 Pass

旧模型是线性的 Phase 链：

```
Phase 0 → Phase 1 → Phase 2 → ... → Phase 6 → 结束
```

新模型：**一切皆 Pass**。Phase 只是 Pass 的预设组合（Auto 模式）。

### Pass 的定义

每个 Pass 是一个独立的、可插拔的处理单元，只声明两件事：

- **requires**: 我需要哪些 artifact 已经存在（前置条件）
- **produces**: 我产出哪些 artifact（后置条件）

Pass 之间**不直接引用彼此**，通过 artifact 间接耦合。这和 Make 的思路一样——target 之间通过文件依赖连接。

### Pass 的属性

```yaml
pass:
  name: "diverge.generate"
  kind: transform              # analysis（只读）或 transform（修改）
  requires: [seed]
  produces: [proposals]
  interaction:
    mode: generative           # generative / adversarial / human-only / automated
    ai_roles: [generator]
    human_role: reviewer
    human_must_act: false
```

### Pass 的粒度判断标准

- 用户会不会想**单独跑**这个 pass？→ 会的话就该独立
- 用户会不会想**替换**这个 pass？→ 会的话就该独立
- 这个 pass 的输入输出能不能**清晰定义**？→ 能的话就该独立

不需要把 pass 拆到原子操作级别（如 format-prompt / call-ai / parse-response），那些是 pass 的内部实现。

### Phase 作为 Auto Preset

Phase 是一组 pass 的快捷方式：

```bash
# Auto 模式（大多数用户）
process diverge              # 等价于运行 diverge.* 的所有 pass

# 手动模式（高级用户）
process pass run diverge.generate
process pass run diverge.validate
process pass run diverge.challenge

# 自定义 Pipeline
process pipeline run my-flow.yaml
```

渐进式复杂度：入门用 Phase → 进阶拆 Pass → 专家自定义 Pipeline。

### Pass 的依赖模型（DAG）

Pass 之间形成有向无环图：

```
seed.init ──→ diverge.generate ──→ converge.analyze ──→ skeleton.generate
                    │                     │
                    ▼                     ▼
             diverge.validate      converge.extract ──→ converge.validate
                    │
                    ▼
             diverge.challenge
```

PassManager 的调度逻辑：

1. 查找目标 pass 组
2. 拓扑排序（按 requires/produces 依赖）
3. 检查前置 artifact 是否存在
4. 依次执行，或并行执行无依赖的 pass
5. 任何 pass 失败 → 停在那里

---

## 三、IR 模型：独立文件 + Manifest

### 设计决定

保持独立 YAML 文件（人类友好），加一个轻量的 manifest 作为"登记簿"。

### Manifest 结构

```yaml
# .process/manifest.yaml
project:
  name: my-app
  created: 2024-01-15
  pack: software              # 使用的领域包

artifacts:
  seed:
    file: seed.yaml
    produced_by: seed.init
    version: 2
    hash: abc123
  proposals:
    file: diverge_summary.yaml
    produced_by: diverge.generate
    version: 1
    depends_on: [seed]

cycles:
  - id: v1-initial
    started: 2024-01-15
    completed: 2024-03-20
    type: greenfield
    artifacts: [seed-v1, proposals-v1, rules-v1, skeleton-v1]

  - id: v2-auth-redesign
    started: 2024-06-01
    type: evolution
    trigger: "用户量超过预期，session 方案扛不住"
    parent: v1-initial
    active: true
```

Manifest 的作用：

- 追踪每个 artifact 的来源、版本、依赖
- 记录决策周期的时间线
- 让 PassManager 判断哪些 pass 需要重新跑（类似 Make 的时间戳机制）

### 目录结构

```
.process/
  manifest.yaml              # 登记簿
  config.yaml                # 项目配置
  current/                   # 当前周期的活跃 artifact
    seed.yaml
    rules.yaml
    decisions/
      active/
      archive/
  archive/                   # 历史周期
    v1-initial/
    v2-auth-redesign/
  knowledge/                 # 跨周期知识库
    heuristics.yaml
    lessons.yaml
  prompts/                   # 项目级 prompt 覆盖
    claude/
    openai/
    _default/
  reports/                   # 生成的可视化产物
```

### Git Track 策略

是否 git track `.process/` 是个人选择，工具不强制：

- **git**: 整个 `.process/` 被 track，团队共享
- **local**: `.process/` 在 `.gitignore` 里，纯个人使用
- **hybrid**: 共享 artifact（rules, skeleton）被 track，个人决策记录不 track

---

## 四、决策周期（Cycle）

### 核心洞察

真实项目不是线性的"走完流程就结束"。一个项目会经历多次决策周期。

**决策周期是一等公民。**

```
一个项目 = 多个决策周期
每个周期 = 一组 pass 的执行
所有周期共享同一个 artifact 仓库和决策历史
```

### 项目生命周期的五个阶段

**1. 诞生（Birth）** — 从零开始或从现有项目 adopt

**2. 成长（Growth）** — 快速迭代，频繁做决策（branch-loop）

**3. 成熟（Maturity）** — 维护模式，守住旧决策，监控偏离

**4. 演化（Evolution）** — 大改/重构，在已有项目上开启新的决策周期

**5. 退役（Sunset）** — 迁移计划、知识归档、经验提取

### 周期之间的关系

Postmortem 不再是终点——它是一个周期的终点，也是下一个周期的输入：

```
Cycle 1 的 postmortem
  → "我们当初选 REST 是错的，应该用 GraphQL"
    → Cycle 2 的 seed 里包含这个教训
      → Cycle 2 的 diverge 会避免重蹈覆辙
```

决策历史变成了项目的"肌肉记忆"。

---

## 五、Adopt：一组独立的 Analysis Pass

### 设计决定

Adopt 不是一个特殊 Phase，而是一组可独立使用的 analysis pass。它们的产出恰好和正向流程兼容，所以可以接入后续流程，但不必须。

### Adopt Pass 清单

| Pass | 产出 | 独立价值 |
|------|------|----------|
| `adopt.scan-structure` | skeleton.yaml | 项目结构可视化 |
| `adopt.infer-conventions` | rules.yaml（推断） | 代码规范报告 |
| `adopt.scan-git-history` | decisions_log.yaml（考古） | 决策历史还原 |
| `adopt.scan-dependencies` | seed.yaml 部分字段 | 技术栈分析 |
| `adopt.gap-analysis` | gap-report.yaml | 识别缺失的决策记录 |

### 关键特性

- 每个 pass 可以单独运行，不依赖其他 adopt pass
- 产出的 artifact 和正向流程的产出**同构**
- adopt 之后可以无缝接入任何后续 pass
- 即使不用 Process CLI 的后续流程，"从 git history 考古决策"本身就有独立价值

---

## 六、知识传递

### 知识的四个层次

| 层次 | 示例 | 保质期 | 价值 |
|------|------|--------|------|
| **事实** | "我们选了 PostgreSQL" | 最长 | 最低 |
| **推理** | "选 PG 因为数据关系复杂且团队熟悉 SQL" | 长 | 高 |
| **教训** | "低估了 PG 的运维成本" | 中 | 很高 |
| **启发式** | "数据库选型时团队熟悉度权重应高于技术优势" | 需验证 | 最高 |

启发式是跨周期积累才能产生的——从多次教训中提炼出的经验法则。

### 三种传递机制

**机制一：显式约束传递**

上一个周期的 rules 自动成为下一个周期的前置约束。除非人类显式废除（需记录原因）。

类比：法律体系——先例约束后续判决，除非明确推翻。

**机制二：上下文注入**

历史决策作为参考注入到新周期的 pass 中（如 diverge.generate 的 prompt），但不强制遵守。

类比：团队里的老人给新人讲历史。

**机制三：启发式提炼**

从多个周期的教训中自动提炼模式，AI 辅助但人类确认。

```
Cycle 1: "低估了运维成本"
Cycle 3: "低估了迁移成本"
Cycle 5: "低估了培训成本"
→ 提炼："团队倾向于低估非功能性成本，评估时应乘以 1.5-2x"
```

### 知识的衰减

知识需要带保质期和前提条件：

```yaml
heuristic:
  content: "小团队优先选单体架构"
  confidence: 0.8
  preconditions:
    - "团队 < 10 人"
    - "产品处于 PMF 探索期"
  revisit_when:
    - "团队超过 10 人"
    - "单体部署时间超过 30 分钟"
  derived_from:
    - { cycle: v1-initial, project: my-app }
    - { cycle: v1-initial, project: another-app }
```

当 `watch.decision-trigger` 检测到 revisit 条件满足时，主动提醒人类重新评估。

### 跨项目知识流动

两个层级：

- **个人级**：`~/.config/process-cli/knowledge/` — 个人从所有项目中积累的经验
- **团队级**：共享仓库 — 团队共同的经验库

Process CLI 不只是陪伴一个项目的生命周期，而是陪伴一个开发者/团队的成长周期。

---

## 七、人-AI 交互模型

### 三种协作模式

| 模式 | 描述 | 适用 Pass |
|------|------|-----------|
| **生成式** | AI 生成，人类审查 | generate 类 |
| **对抗式** | 多个 AI 角色辩论，人类做裁判 | challenge、review 类 |
| **人类主导** | 人类决策，AI 辅助记录 | decide 类 |
| **全自动** | 纯 AI 执行，无需人类参与 | validate 类 |

### Pass 的交互声明

每个 pass 声明自己的交互模式，CLI 据此编排交互方式：

```yaml
# 对抗式 pass
pass: diverge.challenge
interaction:
  mode: adversarial
  ai_roles:
    - attacker              # 找漏洞的 AI
    - defender              # 为提案辩护的 AI
  human_role: judge
  human_must_act: true
  min_rounds: 2

# 生成式 pass
pass: skeleton.generate
interaction:
  mode: generative
  ai_roles: [generator]
  human_role: reviewer
  human_must_act: false
```

### 与 AI Agent 的关系

Process CLI 在 AI Agent **之上**，作为 Agent 的"宪法"。

**Agent 本身是一种特殊的 pass 执行者：**

```yaml
pass: branch.implement
executor: ai-agent
requires: [skeleton, rules, branch-hypothesis]
produces: [code-changes]
constraints:
  - must_follow: rules.yaml
  - scope: branch-hypothesis.yaml
  - on_conflict: pause_and_escalate
```

Agent 被纳入 pass 调度体系——它不是独立的存在，而是流水线上的一个（很强大的）工人。

Process CLI 产出约束，Agent 在约束内自主行动，watch pass 监控 Agent 的产出是否偏离约束。

---

## 八、决策数据模型

### 固定核心 + 自由扩展

决策的 schema 不是固定的。核心字段统一，额外信息因人而异。

**核心字段（所有人都有）：**

```yaml
decision:
  id: d-001
  content: "选择 PostgreSQL"
  reasoning: "数据关系复杂"
  timestamp: 2024-01-15
  status: active              # active / superseded / archived
```

**个人扩展字段（通过配置定义）：**

```yaml
# .process/config.yaml
decision_schema:
  extra_fields:
    - name: risk_level
      type: enum [low, medium, high]
      required: true
    - name: rollback_plan
      type: text
      required: false
    - name: gut_feeling
      type: text
      required: false
```

### 决策的生命周期

```
active → superseded → archived
active → archived（直接废弃）
```

superseded 是关键状态——记录替代关系：

```yaml
decision:
  id: d-001
  content: "使用 REST API"
  status: superseded
  superseded_by: d-007
  superseded_reason: "客户端查询模式变复杂，REST 端点爆炸"

decision:
  id: d-007
  content: "迁移到 GraphQL"
  status: active
  supersedes: d-001
```

这形成**决策演化链**：`d-001 (REST) → d-007 (GraphQL) → d-015 (tRPC)`

每一次替代都有原因，这本身就是知识。

---

## 九、Prompt 系统

### 设计决定

每个 AI provider 一套独立的 prompt 模板。Prompt 从 Rust 代码迁移到 Tera 模板文件。

### 目录结构

```
.process/prompts/
  claude/
    diverge.generate.md.tera
    diverge.challenge.md.tera
    converge.analyze.md.tera
  openai/
    diverge.generate.md.tera
    ...
  _default/                    # 没有专门适配的 provider 用这个
    diverge.generate.md.tera
    ...
```

### 查找顺序（优先级从高到低）

```
项目本地 .process/prompts/{provider}/{pass}.md.tera
→ 领域包 prompts/{provider}/{pass}.md.tera
→ 项目本地 .process/prompts/_default/{pass}.md.tera
→ 领域包 prompts/_default/{pass}.md.tera
→ 内置默认（编译时嵌入）
```

### 模板变量注入

```markdown
你是一个架构师。基于以下需求，生成 {{num_proposals}} 个不同的架构提案。

## 项目需求
{{seed.description}}

## 约束条件
{% for rule in rules %}
- {{rule.content}}
{% endfor %}

## 历史教训
{% for lesson in knowledge.lessons %}
- {{lesson.content}}（置信度: {{lesson.confidence}}）
{% endfor %}
```

---

## 十、决策语言：半结构化规则

### 问题

rules.yaml 里的规则现在是纯自然语言，人类能读懂但机器无法自动检测违反。

### 方案：自然语言 + 结构化标注

不发明新语言，在自然语言规则上逐步添加结构化元数据。

**可检测规则：**

```yaml
- convention: "所有 API 必须有 rate limiting"
  scope: "src/api/**/*.rs"
  detectable: true
  detection:
    pattern: "rate_limit"
    must_exist_in: "every file matching scope"
  severity: hard              # hard = 必须遵守, soft = 建议
```

**不可检测规则：**

```yaml
- convention: "优先使用组合而非继承"
  scope: "src/**/*.rs"
  detectable: false
  review_hint: "检查是否有不必要的 trait 继承链"
  severity: soft
```

**命令式检测（更强大）：**

```yaml
- convention: "测试覆盖率不低于 80%"
  detection:
    command: "cargo tarpaulin --out json | jq '.coverage'"
    threshold: ">= 80"
```

### 渐进式结构化

一开始可以全是 `detectable: false`，随着项目成熟，逐步给重要规则加上检测模式。这把 rules.yaml 变成了一种声明式的约束系统，介于自然语言和代码之间。

---

## 十一、领域包（Domain Pack）

### 概念

领域包 = 一组 pass + 默认 pipeline + prompt 模板 + artifact schema。

```
process-pack-software/         # 软件开发包（内置）
  passes/
  pipelines/
    default.yaml
    startup-fast.yaml
  prompts/
    claude/
    _default/
  schemas/

process-pack-fiction/          # 小说创作包（社区）
  passes/
    worldbuild.generate
    character.arc
    plot.structure
    chapter.review
  pipelines/
    novel.yaml
    short-story.yaml
  prompts/
  schemas/
```

### 使用方式

```bash
process init --pack software
process init --pack fiction
```

---

## 十二、决策质量衡量

### 四个维度

| 维度 | 含义 | 量化方式 |
|------|------|----------|
| **考虑充分性** | 考虑了多少替代方案？有无盲区？ | diverge 产出的提案数、排除理由是否充分 |
| **推理显式性** | 逻辑链是否清晰？假设是否明确？ | decisions_log 的 reasoning 完整度 |
| **预测校准度** | 预测的风险和实际风险有多吻合？ | postmortem 时对比预测 vs 实际 |
| **学习闭环性** | 上次教训有没有被应用？ | 同样的错误是否重复出现 |

### 决策者画像

基于多个周期的数据，生成个人决策画像（作为 report pass 的产物）：

```
考虑充分性: ████████░░ 80%
  强项：总是能想到 3+ 个方案
  弱项：容易忽略运维视角

预测校准度: █████░░░░░ 50%
  强项：性能风险预测准确
  弱项：严重低估集成复杂度（连续 4 个周期）
```

这不是 KPI，而是自我认知工具。

---

## 十三、社区与生态

### 共享内容

社区共享两类东西：

**1. Pass（工具能力）**

- 新的 AI provider 适配
- 特定领域的 review 模板
- 自动化检测 pass

**2. 知识库（决策经验）**

- 领域特定的启发式规则
- Challenge 提醒库（如"微服务的常见陷阱"）
- 决策框架模板

### 插件化的三层方案

| 层级 | 方式 | 特点 |
|------|------|------|
| Layer 1 | 内置 Pass（Rust 编译进二进制） | 核心流程，性能最好 |
| Layer 2 | 脚本 Pass（外部命令 + manifest） | 任何语言，最简单的扩展方式 |
| Layer 3 | 知识库插件（纯数据） | 不含代码，只有 YAML 经验数据 |

Layer 2 通过 stdin/stdout JSON-RPC 通信，类似 git 的子命令发现机制。

Layer 3 是最低门槛的贡献方式——不需要写代码，只需要把自己的经验整理成 YAML。

---

## 十四、最小内核

把所有领域相关的东西抽走后，不可插拔的核心只有五样：

| 组件 | 职责 |
|------|------|
| **Artifact 管理** | 读写文件 + 维护 manifest |
| **Pass 调度器** | 解析依赖、拓扑排序、执行 pass |
| **Pass 注册/发现** | 找到所有可用 pass（内置 + 外部） |
| **Pipeline 引擎** | 读取 pipeline 定义、按顺序调度 pass |
| **CLI 框架** | 命令解析、用户交互 |

**引擎不可插拔，引擎上跑的所有东西都可插拔。**

所有的 diverge、converge、skeleton 都是 pass，都可以被替换。类比：Docker 引擎本身不包含任何应用，它只负责管理容器。

---

## 十五、哲学基础

### 独特性评估

**不独特的部分**：ADR、发散-收敛（设计思维）、Pass/Pipeline（编译器）、知识管理——这些都是成熟概念。

**独特的部分**：

1. **反共识的哲学定位** — 整个行业追求"AI 做更多"，Process CLI 追求"人类做的那部分质量更高"
2. **对抗性协作模型** — 多个 AI 角色辩论，人类做裁判
3. **决策质量的量化** — 不只记录决策，还评估决策过程的质量
4. **跨周期知识演化** — 能积累"决策基因组"的系统

组件不新，组合方式新，哲学立场新。

### 类比：人工生命

- **基因** = 启发式规则（跨项目传递的经验法则）
- **个体** = 单个决策周期
- **自然选择** = postmortem（好的决策模式被保留，坏的被淘汰）
- **突变** = 新项目带来的新经验
- **物种** = 领域包（不同领域演化出不同的决策模式）

区别在于：这里是**有意识的、人类主导的演化**。

### 自指结构

Process CLI 可以用自己来管理自己的开发——工具用自己的输出来改进自己。

### 长期愿景

> 如果 AI 在 5 年后能做出比人类更好的架构决策，Process CLI 的价值是什么？

**A）过程本身有意义。** 围棋 AI 超过人类后，人类还是在下棋。思考本身是愉悦的。

**B）从"训练决策"变成"理解决策"。** 人类不再是决策者，而是决策的理解者和监督者。

两者兼有。

---

## 十六、与现有 ROADMAP 的关系

本文档描述的是**终极愿景**。现有 ROADMAP.md 中的里程碑（MS0-MS9）仍然有效，但需要在以下方向上演进：

- **MS7（架构重构）** 应朝 Pass Engine 方向重构，而非简单的 Command Trait
- **MS9（插件架构）** 应扩展为完整的 Pass 插件体系 + 领域包
- **新增 MS：Adopt Pass 组** — 实现 adopt.* 系列 pass
- **新增 MS：决策周期管理** — 实现多周期支持和知识传递
- **新增 MS：决策语言** — 实现半结构化规则和自动检测

具体的里程碑重排见 ROADMAP.md。
