---
title: Hello, world
description: First post in the Luna static template.
date: 2026-04-01
category: announcements
tags:
  - intro
---

Welcome to the Luna static site template. This page is rendered from
Markdown via [goldmark](https://github.com/yuin/goldmark) and embedded
into a [`templ`](https://templ.guide/) page at build time.

## What this template gives you

- A typed `Post` model parsed from Markdown frontmatter (in `src/lib/`).
- Two reusable layouts (`Base`, `Post`) and a small set of page
  components (`PageHeader`, `ListSection`).
- A simple file router: `src/content/<catalog>/` is a catalog, and any
  top-level `src/content/<name>.md` becomes `/<name>.html`.

## Add another announcement

Drop a new Markdown file in `src/content/posts/` with `category:
announcements` in frontmatter and rebuild. The slug comes from the
filename, so `cool-update.md` becomes `/posts/cool-update/`.
