# Configuration Guide

Process CLI uses a layered configuration system with multiple sources.

## Configuration Precedence

From highest to lowest priority:

1. **Environment Variables** - `PROCESS_CLI_*`, `ANTHROPIC_API_KEY`, etc.
2. **Project Config** - `.process/config.yaml`
3. **Global Config** - `~/.config/process-cli/config.yaml`
4. **Built-in Defaults**

Higher priority sources override lower priority ones.

---

## Configuration File Locations

### Project Configuration
```
<project-root>/.process/config.yaml
```
Project-specific settings. Created automatically by `process init`.

### Global Configuration
```
~/.config/process-cli/config.yaml
```
User-wide defaults. Apply to all projects unless overridden.

---

## Full Configuration Reference

```yaml
# =============================================================================
# AI Provider Configuration
# =============================================================================
ai:
  # Provider selection: auto | claude | openai | ollama | manual
  # "auto" will detect the first available provider in priority order
  provider: auto

  # Claude (Anthropic) configuration
  claude:
    # Model to use (default: claude-sonnet-4-20250514)
    # Can also be set via ANTHROPIC_MODEL env var
    model: claude-sonnet-4-20250514

    # API key (recommended: use ANTHROPIC_API_KEY env var instead)
    api_key: ""

    # Base URL for API (default: https://api.anthropic.com)
    # Can also be set via ANTHROPIC_BASE_URL env var
    base_url: ""

    # Maximum tokens to generate
    max_tokens: 8192

  # OpenAI configuration
  openai:
    # Model to use (default: gpt-4o)
    # Can also be set via OPENAI_MODEL env var
    model: gpt-4o

    # API key (recommended: use OPENAI_API_KEY env var instead)
    api_key: ""

    # Base URL for API (default: https://api.openai.com)
    # Can also be set via OPENAI_BASE_URL env var
    base_url: ""

    # Maximum tokens to generate
    max_tokens: 8192

  # Ollama (local) configuration
  ollama:
    # Model to use
    model: llama3

    # Ollama server endpoint
    endpoint: http://localhost:11434

  # Timeout for AI API calls (seconds)
  timeout_secs: 120

# =============================================================================
# Generator Configuration
# =============================================================================
generators:
  # Git hooks generator settings
  git_hooks:
    pre_commit:
      # Run code formatter check
      run_fmt: true

      # Run linter
      run_lint: true

      # Run tests
      run_tests: true

      # Check for sensitive information (passwords, API keys)
      check_sensitive: true

      # Check for TODO/FIXME comments (warning only)
      check_todo: true

# =============================================================================
# Check Configuration
# =============================================================================
checks:
  # Checks to run for branch gate
  gate_checks:
    - lint
    - test
    - sensitive
    - audit

  # Minimum severity level to fail the gate
  # Options: info | warning | error | critical
  fail_threshold: error

# =============================================================================
# Review Configuration
# =============================================================================
reviews:
  # Default review template
  # Options: general | security | performance | architecture
  default_template: general
```

---

## Environment Variables

### AI Provider Selection
| Variable | Description |
|----------|-------------|
| `PROCESS_CLI_AI_PROVIDER` | Override AI provider (claude, openai, ollama, manual) |

### Claude (Anthropic)
| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Claude API key |
| `ANTHROPIC_MODEL` | Model to use (e.g., claude-sonnet-4-20250514) |
| `ANTHROPIC_BASE_URL` | API base URL |

### OpenAI
| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `OPENAI_MODEL` | Model to use (e.g., gpt-4o) |
| `OPENAI_BASE_URL` | API base URL (for proxies/alternatives) |

---

## Examples

### Minimal Project Config
```yaml
ai:
  provider: claude
```

### Using OpenAI
```yaml
ai:
  provider: openai
  openai:
    model: gpt-4-turbo
```

### Using Local Ollama
```yaml
ai:
  provider: ollama
  ollama:
    model: codellama
    endpoint: http://localhost:11434
```

### Stricter Gate Checks
```yaml
checks:
  gate_checks:
    - lint
    - test
    - sensitive
    - audit
    - coverage
    - todo
  fail_threshold: warning
```

### Relaxed Development Settings
```yaml
generators:
  git_hooks:
    pre_commit:
      run_tests: false  # Skip tests in pre-commit (run in CI)
      check_todo: false

checks:
  fail_threshold: error  # Only fail on errors, not warnings
```

### Security-Focused Configuration
```yaml
checks:
  gate_checks:
    - lint
    - test
    - sensitive
    - audit
  fail_threshold: warning  # Fail on any warning

reviews:
  default_template: security
```

---

## Configuration Tips

### 1. Use Environment Variables for Secrets
Never put API keys in config files. Use environment variables:

```bash
export ANTHROPIC_API_KEY="sk-..."
export OPENAI_API_KEY="sk-..."
```

Or use a `.env` file (not committed to git):
```bash
# .env
ANTHROPIC_API_KEY=sk-...
```

### 2. Global vs Project Config
- Use **global config** for personal preferences (preferred AI model, default settings)
- Use **project config** for project-specific requirements (team standards, CI settings)

### 3. Environment-Specific Overrides
Use environment variables to override settings in different contexts:

```bash
# Development: faster, less strict
PROCESS_CLI_AI_PROVIDER=ollama process check lint

# CI: use Claude API, strict checks
ANTHROPIC_API_KEY=$CI_ANTHROPIC_KEY process check all
```

### 4. Validate Configuration
```bash
# Show current effective configuration
process ai-config show

# Test AI connection
process ai-config test
```

---

## Troubleshooting

### "No AI provider available"
1. Check if API keys are set: `echo $ANTHROPIC_API_KEY`
2. Check if Ollama is running: `curl http://localhost:11434/api/tags`
3. Verify config: `process ai-config show`

### Configuration Not Loading
1. Check file exists: `ls -la .process/config.yaml`
2. Validate YAML syntax: `cat .process/config.yaml | python -c "import yaml, sys; yaml.safe_load(sys.stdin)"`
3. Check for typos in field names

### Environment Variables Not Working
1. Verify export: `env | grep ANTHROPIC`
2. Check shell: environment variables must be exported, not just set
3. Restart terminal if `.bashrc`/`.zshrc` was modified
