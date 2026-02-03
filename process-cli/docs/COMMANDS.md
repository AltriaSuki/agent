# Command Reference

## Phase Commands

### `init`
Initialize a new project.

```bash
process init
```

**Phase:** None → Seed
**Creates:**
- `.process/` directory
- `seed.yaml` template
- `decisions_log.yaml`
- `learnings.yaml`
- `friction.yaml`
- `REJECTED_APPROACHES.md`
- `config.yaml` (AI configuration)

---

### `seed-validate`
Validate the seed.yaml file.

```bash
process seed-validate
```

**Phase:** Seed
**Validates:**
- Required fields: `idea`, `target_user`, `constraints`, `non_goals`, `success_criteria`, `reversibility_budget`
- Non-empty values

---

### `diverge`
Start multi-model divergent exploration.

```bash
process diverge
```

**Phase:** Seed → Diverge
**AI Task:** Generate ≥2 independent technical proposals with:
- Architecture description
- Trade-offs
- Risks
- Constraint alignment check
- Comparison table

**Output:** `diverge_summary.yaml`

---

### `diverge-validate`
Validate diverge output.

```bash
process diverge-validate
```

**Phase:** Diverge
**Validates:**
- At least 2 proposals
- `comparison_dimensions` present

---

### `converge`
Converge proposals, extract rules.

```bash
process converge
```

**Phase:** Diverge → Converge
**AI Task:**
- Select/combine proposals
- Document rejected approaches with reasons
- Extract invariants and conventions

**Output:** `rules.yaml`

---

### `converge-validate`
Validate converge output.

```bash
process converge-validate
```

**Phase:** Converge
**Validates:**
- At least 1 invariant
- `conflict_resolution` policy present

---

### `skeleton`
Generate project skeleton.

```bash
process skeleton
```

**Phase:** Converge → Skeleton
**AI Task:** Generate:
- Directory structure
- Interface definitions
- Rollback template
- Verification checklist

**Output:** `skeleton.yaml`

---

### `skeleton-validate`
Validate skeleton output.

```bash
process skeleton-validate
```

**Phase:** Skeleton
**Validates:**
- Required fields: `directory_structure`, `interfaces`, `rollback_template`, `verification_checklist`
- Git tag `skeleton-v1` exists (warning if not)

---

### `stabilize`
Enter stabilization phase.

```bash
process stabilize
```

**Phase:** Branching → Stabilize
**Actions:**
- Check for unmerged branches (warning)
- Freeze all invariants
- Check high-severity friction points

**Rules during stabilization:**
- No new invariants
- Bugfix branches only
- High-severity friction must be resolved

---

### `postmortem`
Generate retrospective.

```bash
process postmortem
```

**Phase:** Stabilize → Postmortem
**AI Task:** Analyze all artifacts and generate:
- Rules that should have existed earlier
- Rejected approaches review
- Process improvements
- Learnings summary
- Metrics

**Output:** `postmortem.yaml`

---

### `done`
Mark project complete.

```bash
process done
```

**Phase:** Postmortem → Done

---

### `status`
Show current project status.

```bash
process status
```

**Phase:** Any
**Displays:**
- Current phase with progress indicator
- Branch status (if in branching/stabilize phase)
- Artifact file status

---

## Branch Commands

### `branch new <name>`
Create a new branch hypothesis.

```bash
process branch new feature-auth
```

**Phase:** ≥Skeleton
**Creates:** `.process/branches/<name>.yaml` with template:
- `hypothesis`
- `scope.files_to_touch`, `scope.files_not_to_touch`
- `invariants_at_risk`
- `rollback_plan`
- `dependencies`
- `priority`
- `estimated_complexity`
- `status: defined`

---

### `branch start <name>`
Start implementing a branch.

```bash
process branch start feature-auth
```

**Phase:** ≥Skeleton
**Validates:** Hypothesis is filled in
**Actions:**
- Update status: `defined` → `implementing`
- Create git branch `feature/<name>`

---

### `branch review <name>`
Trigger multi-role review.

```bash
process branch review feature-auth
process branch review feature-auth --template security
```

**Phase:** ≥Skeleton
**Options:**
- `--template <name>`: Use specific review template (general, security, performance, architecture)

**AI Task:** Review from 4 perspectives:
1. Security Auditor
2. Performance Engineer
3. User Advocate
4. Maintainability Expert

**Output:** `.process/branches/<name>-review.yaml`
**Status:** `implementing` → `reviewing`

---

### `branch abuse <name>`
Trigger abuse testing.

```bash
process branch abuse feature-auth
```

**Phase:** ≥Skeleton
**AI Task:** Test as malicious user:
- Boundary inputs (empty, long, special chars)
- Unexpected operation sequences
- Resource exhaustion

**Output:** `.process/branches/<name>-abuse.yaml`
**Status:** `reviewing` → `abuse-testing`

---

### `branch gate <name>`
Run merge gate checks.

```bash
process branch gate feature-auth
```

**Phase:** ≥Skeleton
**Checks:**
- Review status
- Abuse test results (no high-severity failures)
- Scope creep (files_not_to_touch)
- Manual confirmations required:
  - Tests pass
  - Invariants verified
  - Rollback tested

---

### `branch merge <name>`
Mark branch as merged.

```bash
process branch merge feature-auth
```

**Phase:** ≥Skeleton
**Status:** → `merged`

---

## Utility Commands

### `learn`
Record a lesson learned.

```bash
process learn "Always validate input before..."
process learn "Cache invalidation is hard" architecture
```

**Phase:** Any
**Arguments:**
- `<lesson>`: The lesson content
- `[category]`: Category (default: general)

**Appends to:** `learnings.yaml`

---

### `friction`
Record a friction point.

```bash
process friction feature-auth "Error messages are confusing"
process friction feature-auth "API is hard to use" high
```

**Phase:** Any
**Arguments:**
- `<branch>`: Branch name
- `<description>`: Friction description
- `[severity]`: low, medium (default), high

**Appends to:** `friction.yaml`

---

## AI Configuration Commands

### `ai-config show`
Show current AI configuration.

```bash
process ai-config show
```

**Displays:**
- Current config file content
- Detected provider

---

### `ai-config set-provider <provider>`
Set AI provider.

```bash
process ai-config set-provider claude
process ai-config set-provider openai
process ai-config set-provider ollama
process ai-config set-provider manual
process ai-config set-provider auto
```

---

### `ai-config test`
Test AI connection.

```bash
process ai-config test
```

**Sends:** Test prompt to verify provider works

---

## Generator Commands

### `generate git-hooks`
Generate Git hooks.

```bash
process generate git-hooks
```

**Output:**
- `.git/hooks/pre-commit`
- `.git/hooks/pre-push`

**Features:**
- Format check (auto-fix)
- Lint check
- Test run
- Sensitive info detection
- TODO/FIXME detection

---

### `generate ci`
Generate CI/CD configuration.

```bash
process generate ci
process generate ci --target github
process generate ci --target gitlab
```

**Options:**
- `--target`: github (default), gitlab, circleci

**Output (GitHub):** `.github/workflows/ci.yml`
**Jobs:**
- Check (fmt, clippy)
- Test (matrix: OS × Rust version)
- Coverage
- Security audit
- Benchmark (main branch only)

---

### `generate makefile`
Generate Makefile.

```bash
process generate makefile
```

**Output:** `Makefile`
**Targets:**
- `check`, `test`, `fmt`, `lint`
- `build`, `release`, `clean`
- `coverage`, `audit`

---

### `generate ide`
Generate IDE configuration.

```bash
process generate ide
process generate ide --target vscode
```

**Output (VS Code):**
- `.vscode/settings.json`
- `.vscode/tasks.json`
- `.vscode/extensions.json`

---

### `generate all`
Generate all applicable configurations.

```bash
process generate all
```

**Runs all generators that are applicable to the current project.**

---

## Check Commands

### `check lint`
Run lint checks.

```bash
process check lint
```

**Rust:** `cargo clippy`
**Auto-fix:** `cargo clippy --fix`

---

### `check test`
Run tests.

```bash
process check test
```

**Rust:** `cargo test`

---

### `check sensitive`
Check for sensitive information.

```bash
process check sensitive
```

**Patterns:**
- Passwords, API keys, secrets, tokens
- AWS credentials
- Private keys
- JDBC connection strings with credentials

---

### `check todo`
Check for TODO/FIXME comments.

```bash
process check todo
```

**Severity:** Info (doesn't fail by default)

---

### `check audit`
Run security audit.

```bash
process check audit
```

**Rust:** `cargo audit`

---

### `check coverage`
Run code coverage.

```bash
process check coverage
```

**Rust:** `cargo tarpaulin`

---

### `check all`
Run all configured checks.

```bash
process check all
```

**Runs:** All checks in `config.checks.gate_checks`

---

## Global Options

```
--verbose, -v       Enable verbose output
--quiet, -q         Suppress non-essential output
--config <path>     Use specific config file
--no-color          Disable colored output
--help, -h          Show help
--version, -V       Show version
```
