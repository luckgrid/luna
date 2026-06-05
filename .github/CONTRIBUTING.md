# Contributing

Thanks for helping improve Luna. This repo is set up for solo devs and small teams — keep changes focused and the bar low.

## Getting started

Run commands from the repository root unless an app README says otherwise.

1. Follow the [Quick Start](../README.md#quick-start): install Proto and Moon, then `moon run luna:install` (or `luna install` once the CLI is on your PATH).
2. Read [AGENTS.md](../AGENTS.md) for workspace layout and stack-specific guardrails.

## Before you push

```bash
luna check          # lint + format:check + typecheck (all stacks)
luna fix            # auto-fix where supported
luna test           # run application tests
```

For a narrower scope: `luna check app`, `luna test --affected`, etc.

## Commits and pull requests

- Keep commits and PRs focused on one change when you can.
- Fill in the PR template — a short summary and bullet list is enough.
- Link related issues if there are any.

## Questions

Open a [GitHub issue](https://github.com/luckgrid/luna/issues) or see [SUPPORT.md](SUPPORT.md).
