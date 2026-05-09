---
title: Getting started
description: How to build and serve the Luna static site locally.
date: 2026-04-02
category: guides
tags:
  - guide
---

The `web` app is a **Bun** static site generator. Markdown bodies use
[`Bun.markdown`](https://bun.com/docs/runtime/markdown); HTML layouts and
partials live in `src/templates/` and are filled by code in `src/lib/`.

## Build

```sh
moon run web:build
```

Runs `bun ./src/main.ts`, then **Vite** to compile `src/styles.css`
(with `@luna/ds` and Tailwind) into `dist/styles.css`.

## Develop

```sh
moon run web:dev
```

Runs **`vite build --watch`** for CSS and watches `src/content/`,
`src/lib/`, and `src/templates/`; serves `dist/` on `WEB_PORT` (default `3000`).

## Add content

- **Catalog post** — add `src/content/posts/<slug>.md` with optional
  `category:` in frontmatter. The filename sets the slug.
- **Single page** — add `src/content/<name>.md`; it becomes `/<name>.html`
  (for example `legal.md` → `/legal.html`).
