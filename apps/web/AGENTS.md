# Web App Agent Guide

Companion to [`README.md`](README.md). Read README first for stack, features, and workflow.

## Core Rules

- Follow root repo rules — see [root AGENTS.md](../../AGENTS.md).
- Run commands from the workspace root unless this README says otherwise.
- Keep resource links in README, not here.

## Layout Rules

- **Dispatcher pattern**: use root `page.html` / `section.html` to branch on `params.layout`. Do not create `layouts/_default/`, `layouts/article/`, `layouts/catalog/`, or `layouts/collection/`.
- **Do not add** `single.html` — `page.html` handles `kind=page`.
- **Partials**: use flat `_partials/*.html` fragments for shared markup. Do not add nested folders like `_partials/shell/` or `_partials/ui/` unless Hugo requires it.
- **Collections**: follow the locked DOM structure (D4) for sidebar + TOC. See README features section.
- **Marketing copy**: place in content files, not hard-coded in templates.

## Adding New Patterns

- Add new `params.layout` values by editing `page.html` or `section.html` + update matching archetype.
- Add new render hooks under `_markup/`. Language-specific hooks take precedence over the generic fallback.
- Keep high-level feature additions in README; do not document implementation details here.

## Validation

Before committing layout or archetype changes:

```sh
cd apps/web && hugo --gc --minify
```

Ensure builds pass and review template metrics.
