---
title: Getting started
description: How to build and serve the Luna static site locally.
date: 2026-04-02
category: guides
tags:
  - guide
  - dev setup
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

Shared markup lives as flat files under **`layouts/_partials/`** (e.g. `brand.html`, `hero.html`, `article-collection.html`). Page and section **layouts** are inlined in root [`page.html`](../../layouts/page.html) and [`section.html`](../../layouts/section.html) (branch on **`params.layout`**); [`home.html`](../../layouts/home.html) matches the **`simple`** page shape.

- **Posts catalog** — [`/posts/`](./) → `section.html` → **`catalog`** branch
- **New post** — `hugo new content posts/<slug>.md -k article`
- **New legal policy** — `hugo new content legal/<slug>.md -k article`
- **New page inside a collection hub** — `hugo new content posts/list-example/<slug>.md -k article`
- **New collection** — `hugo new content <section>/_index.md -k collection`, then adjust cascade and copy in the generated front matter if needed.
- **`toc`** — shown automatically when a page has at least two `##` headings (override with `toc: true` / `toc: false` in front matter). Applies to collection pages (right **`aside[data-toc]`** inside **`<main>`**) and article pages (TOC **`aside`** after **`<article>`**). Layout chrome is driven by **`type`** on the page (and cascade on section `_index` files).
