# Skills Compatibility

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

OpenClaw's skills system provides reusable agent capabilities. openclaw-rs supports the core skills format.

---

## What Are Skills?

Skills are predefined capabilities that agents can use. They're like macros or shortcuts for common operations.

In OpenClaw:
```json5
{
  "skills": {
    "summarize": {
      "description": "Summarize text or files",
      "prompt_template": "Summarize the following: {{input}}"
    }
  }
}
```

---

## Compatibility Status

| Feature | Status | Notes |
|---------|--------|-------|
| Basic skill definitions | ✅ | Full support |
| Prompt templates | ✅ | Mustache-style |
| Tool-based skills | ✅ | Single tool |
| Multi-step skills | ⚠️ | Partial |
| Skill chaining | ⚠️ | Limited |
| Custom skill handlers | ⚠️ | Via IPC bridge |

---

## Supported Skill Types

### Prompt Skills

Skills that apply a prompt template:

```json5
{
  "skills": {
    "explain_code": {
      "description": "Explain code in simple terms",
      "prompt_template": "Explain this code as if to a beginner:\n\n{{code}}"
    }
  }
}
```

**Usage:**
```
/explain_code

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

### Tool Skills

Skills that execute a tool:

```json5
{
  "skills": {
    "list_files": {
      "description": "List files in a directory",
      "tool": "bash",
      "command": "ls -la {{path}}"
    }
  }
}
```

### Composite Skills

Skills combining prompts and tools:

```json5
{
  "skills": {
    "analyze_file": {
      "description": "Read and analyze a file",
      "steps": [
        {
          "tool": "file_read",
          "input": { "path": "{{file}}" }
        },
        {
          "prompt": "Analyze this file content:\n\n{{step_1_result}}"
        }
      ]
    }
  }
}
```

⚠️ Multi-step skills have partial support.

---

## Defining Skills

### In Configuration

```json5
// ~/.openclaw/openclaw.json
{
  "skills": {
    "my_skill": {
      "description": "What this skill does",
      "prompt_template": "Template with {{variables}}"
    }
  }
}
```

### In Separate Files

```json5
// ~/.openclaw/skills/review.json5
{
  "name": "code_review",
  "description": "Review code for issues",
  "prompt_template": "Review this code for bugs, security issues, and improvements:\n\n{{code}}"
}
```

Load with:
```json5
{
  "skills": {
    "code_review": {
      "file": "~/.openclaw/skills/review.json5"
    }
  }
}
```

---

## Using Skills

### Via CLI

```bash
# List available skills
openclaw skills list

# Show skill details
openclaw skills show code_review
```

### Via Chat

In the web dashboard or API, use `/skill_name`:

```
/code_review

function add(a, b) {
  return a + b
}
```

### Via API

```javascript
// POST /api/chat
{
  "message": "/code_review\n\nfunction add(a, b) { return a + b }",
  "session_id": "..."
}
```

---

## Template Variables

Skills use Mustache-style templates:

| Variable | Description |
|----------|-------------|
| `{{input}}` | User input after skill name |
| `{{file}}` | File path (if applicable) |
| `{{selection}}` | Selected text |
| `{{context}}` | Current context |

### Example

```json5
{
  "skills": {
    "translate": {
      "description": "Translate text to another language",
      "prompt_template": "Translate the following to {{language}}:\n\n{{text}}"
    }
  }
}
```

Usage:
```
/translate language=Spanish text=Hello, how are you?
```

---

## Built-in Skills

openclaw-rs includes some built-in skills:

| Skill | Description |
|-------|-------------|
| `help` | Show available commands |
| `clear` | Clear conversation |
| `export` | Export conversation |
| `status` | Show system status |

---

## OpenClaw Skill Compatibility

### Supported

- Basic prompt templates
- Single-tool skills
- Variable substitution
- Skill descriptions

### Partial Support

- Multi-step skills (basic)
- Skill parameters
- Conditional logic

### Not Yet Supported

- Complex skill chaining
- External skill repositories
- Skill versioning

---

## Migrating Skills

If you have existing OpenClaw skills:

1. Copy skill definitions:
   ```bash
   cp -r ~/.config/openclaw/skills ~/.openclaw/skills
   ```

2. Update config to reference them:
   ```json5
   {
     "skills_dir": "~/.openclaw/skills"
   }
   ```

3. Test each skill:
   ```bash
   openclaw skills validate
   ```

---

## Creating Compatible Skills

To create skills that work with both projects:

```json5
{
  "name": "my_skill",
  "description": "Works with OpenClaw and openclaw-rs",
  "version": "1.0.0",

  // Basic format - maximum compatibility
  "prompt_template": "Simple template: {{input}}"
}
```

Avoid:
- Complex multi-step workflows
- External dependencies
- Custom handlers (use IPC bridge instead)

---

## Next Steps

[:material-power-plug: Plugin Compatibility](plugins.md){ .md-button .md-button--primary }
[:material-swap-horizontal: Migration Guide](migration.md){ .md-button }
