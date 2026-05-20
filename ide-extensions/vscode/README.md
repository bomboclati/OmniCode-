# OmniCode VS Code Extension

Connect VS Code to the OmniCode autonomous AI coding agent.

## Features

- Chat with OmniCode directly from VS Code
- Run swarm mode for parallel AI processing
- Review code with AI
- View diffs from agent changes
- Deploy projects
- Semantic code search
- Self-healing tests
- Documentation generation

## Requirements

- OmniCode server running (`omni serve` or `omnicode serve`)
- Default port: 9421

## Commands

| Command | Description |
|---------|-------------|
| `OmniCode: Chat with Agent` | Open the OmniCode chat panel |
| `OmniCode: Command Palette` | Open the command palette |
| `OmniCode: Run Swarm` | Start swarm mode |
| `OmniCode: Review Code` | Review current file |
| `OmniCode: Deploy` | Deploy the project |
| `OmniCode: Semantic Search` | Search codebase |
| `OmniCode: Self-Heal Tests` | Auto-fix test failures |
| `OmniCode: Generate Docs` | Regenerate documentation |

## Configuration

```json
{
    "omnicode.port": 9421
}
```

## Development

1. `npm install`
2. `npm run compile`
3. Press F5 to start debugging
