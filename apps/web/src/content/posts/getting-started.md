---
title: Getting started
description: How to build and serve the Luna static site locally.
date: 2026-04-02
category: guides
tags:
  - guide
---

The `web` app is a Go + [templ](https://templ.guide/) static site
generator that renders Markdown content into reusable layout templates.

## Build

```sh
moon run web:build
```

This runs `go tool templ generate`, builds the Tailwind CSS bundle, then
runs the SSG to produce `dist/` with all pages and assets.

## Develop

```sh
moon run web:dev
```

Watches templ files, regenerates Go bindings, rebuilds the site, and
serves `dist/` on `WEB_PORT` (default `3000`).

## Add content

- **Catalog post** — drop a `.md` file in `src/content/posts/` with
  `category: <name>` in frontmatter. The filename becomes the slug.
- **Single page** — drop a `.md` file at `src/content/<name>.md`. It is
  rendered as `/<name>.html` using its route template (for example,
  `src/pages/legal.templ` for `src/content/legal.md`).
