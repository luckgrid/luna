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

| Kind            | File                                                | When to use                                                                              |
| --------------- | --------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **default**     | [`default.md`](../../archetypes/default.md)         | Generic pages — `hugo new content <path>`                                                |
| **catalog**     | [`catalog.md`](../../archetypes/catalog.md)         | Section **catalog** index (list + search + groups), e.g. another **`/posts/`**-style hub |
| **article**     | [`article.md`](../../archetypes/article.md)         | Single **articles**: posts, legal policies, `posts/list-example/…`                       |
| **article-hub** | [`article-hub.md`](../../archetypes/article-hub.md) | **Article hub** (`_index` with collection sidebar + TOC), e.g. `legal`, `list-example`   |

Shared markup lives in **`layouts/_partials/`** (`catalog-main`, `article-hub-main`, `article-single`, `article-card`, `articles-featured`, …). Root [`baseof.html`](../../layouts/baseof.html), [`list.html`](../../layouts/list.html), and [`single.html`](../../layouts/single.html) branch on **content path / section** so you do not add a new `layouts/<section>/` tree for every area (see [app README](../../../README.md)).

- **Posts catalog** — [`/posts/`](./) → [`catalog-main`](../../layouts/_partials/catalog-main.html) via [`list.html`](../../layouts/list.html).
- **New post** — `hugo new content posts/<slug>.md -k article`
- **New legal policy** — `hugo new content legal/<slug>.md -k article`
- **New page inside a collection hub** — `hugo new content posts/list-example/<slug>.md -k article`
- **New article hub** — `hugo new content <section>/_index.md -k article-hub`, then extend the **`$collection`** conditions in `baseof.html`, `list.html`, and `article-single.html` (same pattern as `legal` / `posts/list-example`).
- **`toc`** — single flag for both collection pages (right **`aside[data-toc]`** next to **`div[data-layout="collection"]`**) and **`layout: article`** singles (TOC **`aside`** after **`<article>`** inside **`<main>`**). For collections, `<body>` only sets **`data-pattern`**; the grid shell is **`div[data-layout="collection"]`**. For **`article`** pages, set **`layout`** / **`pattern`** on `<body>` to match **[@luna/ds](../../../../../packages/ds/src/layouts/article.css)**.
