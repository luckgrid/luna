# `web`

Luna static marketing/content site: Hugo, Goldmark Markdown, Go templates, and `@luna/ds` styles via Tailwind CSS v4.

## Purpose

The static site for Luna's marketing pages, blog posts, legal docs, and collections. Serves as the public-facing complement to the interactive app (`apps/app`). Output goes to **`dist/`** for deployment to any static host.

## Stack

- 📰 [Hugo](https://gohugo.io/) — static site generator; Markdown to HTML, RSS, sitemap
- ✍️ [Goldmark](https://github.com/yuin/goldmark/) — CommonMark Markdown parser
- 🧩 [Go html/template](https://gohugo.io/templates/) — layouts, partials, shortcodes
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) — utility CSS via `@luna/ds`
- 🖍️ [Chroma](https://github.com/alecthomas/chroma) — syntax highlighting for code blocks
- 🟢 [Bun](https://bun.sh/) — runs Tailwind CLI for CSS compilation
- 🐹 [Go](https://go.dev/) — runtime for Hugo CLI (via `go tool`)

See root [README Tech Stacks](../../README.md#tech-stacks) for toolchain details.

## Features

- **Home, pages, sections** — standard Hugo content types
- **Article layouts** — posts, legal policies, standalone pages
- **Catalog sections** — searchable article indexes (e.g. `/posts/`)
- **Collections** — multi-page groups with sidebar navigation and optional TOC
- **Taxonomies** — tags and categories (rendered to `/tags/`, `/categories/`)
- **Archetypes** — content starters: `default`, `catalog`, `article`, `collection`
- **Dispatcher layouts** — `page.html` and `section.html` branch on `params.layout`
- **Markdown render hooks** — links, images, headings, blockquotes, code blocks
- **Shortcodes** — reusable content components
- **SEO** — meta, canonical, Open Graph, RSS

## Local Development

From the workspace root:

```sh
moon run web:dev
```

From this directory:

```sh
bun run dev
```

Default port: `WEB_PORT` (`3001`).

## Build and run

From the workspace root:

```sh
moon run web:build
moon run web:start
```

From this directory:

```sh
bun run build
bun run start
```

Build output: **`dist/`** (gitignored at repo root).

## App Configs

- project config: [`moon.yml`](moon.yml)
- site config: [`hugo.toml`](hugo.toml)
- Hugo CLI: [`go.mod`](go.mod) (`go tool hugo`)
- CSS build: [`package.json`](package.json)
- environment: root [`.env.local`](../../.env.local)

## Environment Variables

| Variable   | Description             | Default |
| ---------- | ----------------------- | ------- |
| `WEB_PORT` | Dev / preview bind port | `3001`  |

## Project Structure

```text
web/
├── src/
│   ├── archetypes/        # Content starters
│   ├── assets/            # CSS and static assets
│   ├── content/           # Markdown source
│   ├── data/              # JSON/TOML/YAML data
│   ├── i18n/              # Translations
│   └── layouts/           # Go templates
├── public/                # Static files (favicon, robots.txt)
└── dist/                  # Build output
```

See [`AGENTS.md`](AGENTS.md) for layout rules and editing guardrails.
