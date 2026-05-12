---
title: Getting started
description: How to build and serve the Luna static site locally.
date: 2026-04-02
category: guides
tags:
  - guide
---

The `web` app is a **Hugo** static site. Markdown uses [Goldmark](https://github.com/yuin/goldmark/); templates live in `src/layouts/` (mounted as Hugo’s `layouts/`).

## Build

```sh
moon run web:build
```

Runs `go tool hugo --gc --minify` (pinned in `go.mod`) with [Tailwind v4](https://gohugo.io/functions/css/tailwindcss/) processing `src/assets/css/main.css` (which imports `@luna/ds`) into `dist/`.

## Develop

```sh
moon run web:dev
```

Runs **`go tool hugo server`** with live reload; set `WEB_PORT` (default `3001`).

## Add content

Archetypes only apply to **`hugo new content`**. This repo uses four **`-k`** kinds ([archetypes](https://gohugo.io/content-management/archetypes/)):

- **`default`** -> [`default.md`](../../archetypes/default.md): generic pages via `hugo new content <path>`
- **`catalog`** -> [`catalog.md`](../../archetypes/catalog.md): section **catalog** indexes, like another `/posts/`-style hub
- **`article`** -> [`article.md`](../../archetypes/article.md): single **articles** such as posts, legal policies, and `posts/list-example/...`
- **`collection`** -> [`collection.md`](../../archetypes/collection.md): collection `_index` pages with shared sidebar + optional TOC

Shared markup lives in **`layouts/_partials/`** (brand, nav, breadcrumbs, TOC, cards, featured posts, collection sidebar). Hugo resolves the main page shells through [`home.html`](../../layouts/home.html), [`page.html`](../../layouts/page.html), [`catalog/section.html`](../../layouts/catalog/section.html), [`collection/section.html`](../../layouts/collection/section.html), [`collection/page.html`](../../layouts/collection/page.html), and [`article/page.html`](../../layouts/article/page.html).

- **Posts catalog** — [`/posts/`](./) → [`catalog/section.html`](../../layouts/catalog/section.html)
- **New post** — `hugo new content posts/<slug>.md -k article`
- **New legal policy** — `hugo new content legal/<slug>.md -k article`
- **New page inside a collection hub** — `hugo new content posts/list-example/<slug>.md -k article`
- **New collection** — `hugo new content <section>/_index.md -k collection`, then adjust `params.pattern` and copy in the generated front matter if needed.
- **`toc`** — single flag for both collection pages (right **`aside[data-toc]`** inside the collection **`<main>`**) and article pages (TOC **`aside`** after **`<article>`** inside **`<main>`**). Body hooks now come from front matter **`params.layout`** / **`params.pattern`** rather than path-based conditions in the root layouts, and collection child pages keep the same two-sidebar shell.
