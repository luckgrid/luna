---
title: Hello, world
description: First post in the Luna static template.
date: 2026-04-01
category: announcements
tags:
  - intro
  - hello world
---

Welcome to the Luna static site template. This page is rendered from Markdown via [Goldmark](https://gohugo.io/getting-started/configuration-markup/) and composed with Go templates under `src/layouts/`.

## What this template gives you

- A minimal Hugo project with `@luna/ds` + Tailwind v4 (CLI + Hugo asset pipeline).
- Layouts for home, post catalog, articles, collection-style Legal / example nested sections, and SEO/RSS.

## Add another announcement

Add a new Markdown file in `src/content/posts/` with `category: announcements` in frontmatter and rebuild. The slug comes from the filename, so `cool-update.md` becomes `/posts/cool-update/`.
