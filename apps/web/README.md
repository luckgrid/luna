# `web`

Luna **marketing / content** static site: [Hugo](https://gohugo.io/about/introduction/), [Goldmark](https://gohugo.io/getting-started/configuration-markup/) Markdown, [Go html/template](https://gohugo.io/templates/) layouts, and **[`@luna/ds`](../../packages/ds/README.md)** styles. A **Tailwind CSS v4** build step (`@tailwindcss/cli`) emits `src/assets/css/bundle.css`; Hugo fingerprints and ships it in **`dist/`** (see [Configuration](#configuration)).

Source lives under **`src/`** (content, layouts, processed assets); **`public/`** at the project root is the Hugo **static** dir (favicon, `robots.txt`, …); **`dist/`** is the build output ([`hugo.toml`](hugo.toml) module mounts map `src/*` onto Hugo’s conventional folders).

This document is the **single entry point** for this app. Monorepo orchestration and quality gates live in the [root README](../../README.md).

---

## Overview

| Item                    | Detail                                                                                                                                                                                                                                                                                                                                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Generator**           | [Hugo](https://gohugo.io/documentation/) — static HTML, RSS, and sitemap                                                                                                                                                                                                                                                                                      |
| **Markdown**            | [Goldmark](https://github.com/yuin/goldmark/) (default) — CommonMark + GFM-style features via [config](https://gohugo.io/getting-started/configuration-markup/)                                                                                                                                                                                               |
| **Syntax highlighting** | [Chroma](https://github.com/alecthomas/chroma) via [`markup.highlight`](https://gohugo.io/getting-started/configuration-markup/#highlight)                                                                                                                                                                                                                    |
| **Styles**              | `@luna/ds` → Tailwind v4 CLI — **no Sass**                                                                                                                                                                                                                                                                                                                    |
| **Hugo CLI**            | Pinned in [`go.mod`](go.mod) as a **[`go tool`](https://go.dev/doc/go1.24#tools)** (`github.com/gohugoio/hugo`); Moon runs **`go tool hugo …`**. **Go** is pinned in [`.prototools`](../../.prototools) (proto) — same role as Python for `apps/api`. **Git** helps [Hugo Modules](https://gohugo.io/hugo-modules/configuration/) and themes from Git remotes |
| **Package manager**     | [Bun](https://bun.sh/) (installs Tailwind CLI for this app)                                                                                                                                                                                                                                                                                                   |

---

## Features

**Layout terms:** **catalog** = searchable section index of articles (e.g. `/posts/`); **collection** = multi-page group with shared chrome (sidebar + optional right TOC), e.g. [`legal/`](src/content/legal/) or nested [`posts/list-example/`](src/content/posts/list-example/) (Legal is a collection, not under the posts catalog); **article** = one Markdown page (standalone, in a catalog, or inside a collection).

- Content-driven **home**, **posts** catalog with **category** grouping (nested **`posts/list-example/`** posts are excluded from that catalog), and per-post pages.
- **Collection sections** — [`legal/`](src/content/legal/) and [`posts/list-example/`](src/content/posts/list-example/) share [`src/layouts/_partials/article/collection.html`](src/layouts/_partials/article/collection.html): filter-as-you-type sidebar, optional right **TOC** via [`TableOfContents`](https://gohugo.io/methods/page/tableofcontents/) when **`toc: true`** on the page, and an article list in the main column. Collection routes carry **`data-layout="collection"`** and a section-specific **`data-pattern`** on `<body>` (see [`main.css`](src/assets/css/main.css)).
- **[Archetypes](https://gohugo.io/content-management/archetypes/)** — four starters (**`default`**, **`catalog`**, **`article`**, **`collection`**) with **`-k`**; see [Archetypes](#archetypes-hugo-new) below.
- **Shared article UI** — Hugo template types drive the shells: root [`home.html`](src/layouts/home.html) / [`page.html`](src/layouts/page.html) / [`section.html`](src/layouts/section.html), plus [`catalog/section.html`](src/layouts/catalog/section.html), [`collection/section.html`](src/layouts/collection/section.html), and [`article/page.html`](src/layouts/article/page.html) with smaller partials under [`src/layouts/_partials/article/`](src/layouts/_partials/article/) for cards, nav, breadcrumbs, TOC, and featured posts ([Hugo templates](https://gohugo.io/templates/)).
- **SEO**: meta description, canonical URL, minimal Open Graph / Twitter tags grouped in [`src/layouts/_partials/head/metadata.html`](src/layouts/_partials/head/metadata.html); [sitemap](https://gohugo.io/templates/sitemap-template/), [RSS](https://gohugo.io/templates/rss/).
- **Shortcodes** — e.g. [`src/layouts/_shortcodes/alert.html`](src/layouts/_shortcodes/alert.html) (`{{< alert >}}...{{< /alert >}}`).
- **Taxonomy listing disabled** — `disableKinds = ["taxonomy", "term"]` in [`hugo.toml`](hugo.toml) so `tags:` in front matter stay as metadata without generating `/tags/` pages (adjust if you want tag URLs).

---

## Project structure

```text
web/
  go.mod / go.sum        Hugo CLI version (`tool github.com/gohugoio/hugo`) + module sums for `go tool`
  hugo.toml              Site config, publishDir, module mounts (src → Hugo dirs), staticDir
  package.json           workspace:* @luna/ds + tailwindcss CLI
  public/                Static files published verbatim (favicon, robots.txt → site root)
  dist/                  Production output (gitignored at repo root)
  src/
    content/             Markdown source
    layouts/             Go templates (`_partials/`, `_shortcodes/` — see [Hugo templates](https://gohugo.io/templates/))
    assets/
      css/
        main.css         Tailwind entry (@import @luna/ds; @source …)
        bundle.css       Generated by Tailwind CLI (gitignored) — do not commit
    data/                Optional JSON/TOML/YAML for `site.Data`
    archetypes/          `default`, `catalog`, `article`, `collection` — `hugo new content` starters
    i18n/                Optional translation bundles
```

Layouts follow Hugo’s **[new template system](https://gohugo.io/templates/new-templatesystem-overview/)** (v0.146+): root [`baseof.html`](src/layouts/baseof.html) is document chrome only, [`home.html`](src/layouts/home.html) handles the home **Kind**, root [`page.html`](src/layouts/page.html) and [`section.html`](src/layouts/section.html) provide the generic fallbacks, and type-based templates live under [`catalog/`](src/layouts/catalog/), [`collection/`](src/layouts/collection/), and [`article/`](src/layouts/article/). Body hooks now come from front matter **`params.layout`** / **`params.pattern`** so templates can stay aligned with Hugo lookup instead of branching on file paths. Partials live in **`layouts/_partials/`**; shortcodes in **`layouts/_shortcodes/`** ([introduction](https://gohugo.io/templates/introduction/), [templates index](https://gohugo.io/templates/)). For SEO head ideas see [PaperMod](https://github.com/adityatelange/hugo-PaperMod/) (not vendored).

Key partials:

- [`header.html`](src/layouts/_partials/header.html): shared top header for **catalog** and **collection** pages
- [`article/header.html`](src/layouts/_partials/article/header.html): article-page header with logo-only home link + breadcrumbs
- [`article/collection.html`](src/layouts/_partials/article/collection.html): collection navigation with filter-as-you-type search
- [`toc.html`](src/layouts/_partials/toc.html): reusable “On this page” aside for collection and article layouts
- [`article/card.html`](src/layouts/_partials/article/card.html): card link for a page or collection index in catalogs, collections, and featured-post sections
- [`article/featured.html`](src/layouts/_partials/article/featured.html): home **latest** list (`dict` with `title`, `pages`)

**Scaling:** add another **collection** subtree under `content/` by creating its `_index.md` with **`type: collection`**, collection body hooks in **`params`**, and a **`cascade`** block for child articles. Add another searchable article index by creating a new section `_index.md` with **`type: catalog`**.

---

## Prerequisites

From the **repository root**:

1. **[proto](https://moonrepo.dev/docs/proto)** — `bun run setup` or `proto install` (installs **go** per [`.prototools`](../../.prototools); root **`setup`** runs **`moon run web:setup`** which downloads modules for this app).
2. **Bun workspaces** — `bun install` (links `@luna/ds` and installs Tailwind CLI for this app).

**Git** is useful for [Hugo Modules](https://gohugo.io/hugo-modules/configuration/) and for themes installed as submodules. In CI, use the **Go** version from `.prototools` so `go tool hugo` matches local builds.

---

## Development

```sh
moon run web:dev
```

Runs an initial Tailwind compile, then **`tailwindcss --watch`** in the background, then **`go tool hugo server`** on **`0.0.0.0`** with port **`WEB_PORT`** (default **`3001`**). Moon may load [`.env.local`](../../.env.local) (see [`go-web.yml`](../../.moon/tasks/go-web.yml)).

Edit **copy** in [`src/content/`](src/content/), **structure** in [`src/layouts/`](src/layouts/), **site knobs** in [`hugo.toml`](hugo.toml).

---

## Build

```sh
moon run web:build
```

1. `web:setup` (`go mod download`) then `bun install` (via `web:install`).
2. `bunx @tailwindcss/cli -i ./src/assets/css/main.css -o ./src/assets/css/bundle.css --minify`
3. `go tool hugo --gc --minify` → **`dist/`**

Production CSS is loaded from **`src/assets/css/bundle.css`** ([`src/layouts/_partials/head/css.html`](src/layouts/_partials/head/css.html)). If that file is missing, Hugo warns — run the Tailwind step first (Moon always runs it before `go tool hugo`).

### Why not Hugo’s `css.TailwindCSS` only?

The **Tailwind CLI** runs in **Bun** with full **`node_modules`** resolution for `@luna/ds`, matching the [Tailwind v4 + monorepo](https://gohugo.io/functions/css/tailwindcss/) story without fighting bundled resolution inside Hugo alone. See also: [Hugo TailwindCSS](https://gohugo.io/functions/css/tailwindcss/) and [`css.Build`](https://gohugo.io/functions/css/build/) for alternate single-binary pipelines on newer Hugo editions.

---

## Content

### Archetypes (`hugo new`)

Starters in [`src/archetypes/`](src/archetypes/) map to **`-k`** ([archetypes](https://gohugo.io/content-management/archetypes/)):

Supported archetype kinds:

- **`default`** -> [`default.md`](src/archetypes/default.md): generic pages. Example: `hugo new content about.md`
- **`catalog`** -> [`catalog.md`](src/archetypes/catalog.md): section **catalog** index with search + pagination, like **`/posts/`**
- **`article`** -> [`article.md`](src/archetypes/article.md): single **article** such as a post, legal policy, or page under a collection
- **`collection`** -> [`collection.md`](src/archetypes/collection.md): **collection** `_index` with intro copy and **`cascade`** sidebar labels

Tune **`cascade`** / **`toc`** / **`weight`** on real hubs after scaffolding. **`collection.md`** includes a starter **`cascade.params`** block—replace labels per section.

| Role                      | Path                                                                 | Output                                       |
| ------------------------- | -------------------------------------------------------------------- | -------------------------------------------- |
| Home                      | [`src/content/_index.md`](src/content/_index.md)                     | `/`                                          |
| Posts index               | [`src/content/posts/_index.md`](src/content/posts/_index.md)         | `/posts/`                                    |
| Post                      | `src/content/posts/<slug>.md`                                        | `/posts/<slug>/`                             |
| List example (collection) | [`src/content/posts/list-example/`](src/content/posts/list-example/) | `/posts/list-example/` (hub + nested posts)  |
| Legal (collection)        | [`src/content/legal/`](src/content/legal/)                           | `/legal/` (hub), `/legal/<slug>/` (policies) |

### Collection layout (article hubs: legal & list-example)

**`legal`** and **`posts/list-example`** are **collections**: same navigation partial ([`article/collection.html`](src/layouts/_partials/article/collection.html)) with labels driven by **`cascade`** `params` on each section’s `_index.md` (`collection_sidebar_search_label`, `collection_nav_aria`). The **collection shell** is rendered by [`collection/section.html`](src/layouts/collection/section.html), while child pages preserve that same shell through [`collection/page.html`](src/layouts/collection/page.html) or the collection-aware branch in [`article/page.html`](src/layouts/article/page.html). Optional right column TOC appears when **`toc: true`** (renders as **`aside[data-toc]`**).

- **Legal hub** — [`src/content/legal/_index.md`](src/content/legal/_index.md); policies as `src/content/legal/<slug>.md` with **`weight`** for sidebar order.
- **List-example hub** — [`src/content/posts/list-example/_index.md`](src/content/posts/list-example/_index.md); new hubs: **`hugo new content …/_index.md -k collection`**. Pages inside the hub: **`hugo new content posts/list-example/<slug>.md -k article`** ([`article.md`](src/archetypes/article.md)).

Search matches **title**, **description**, and URL path segments (client-side; no external search service).

### Front matter (common)

Common front matter:

- `title`: page heading / `<title>`
- `description`: meta description
- `date`: post date (`YYYY-MM-DD`)
- `category`: display metadata for posts
- `tags`: display-only metadata (no `/tags/` URLs by default)
- `slug`: optional URL slug override
- `weight`: sidebar order in **collection** sections (`legal`, `posts/list-example`, ...)
- `toc`: if true, render right-hand **`aside[data-toc]`** in **collection** and **article** templates
- `params.layout` / `params.pattern`: passed to **`<body>`** as **`data-layout`** / **`data-pattern`**; **`article`**, **`catalog`**, and **`collection`** archetypes align these with the template families and **[@luna/ds article layout](../../packages/ds/src/layouts/article.css)**
- `latest_posts_title`: home-page featured section heading

---

## Configuration

Key files: [`go.mod`](go.mod) (Hugo `go tool`), [`hugo.toml`](hugo.toml)

| Area     | Keys / notes                                                                                                                 |
| -------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Output   | `publishDir = "dist"`                                                                                                        |
| Static   | `staticDir = "public"` — files copied to site root in **`dist/`**                                                            |
| Layout   | `[module.mounts]` — `src/content`, `src/layouts`, `src/assets`, `src/data`, `src/archetypes`, `src/i18n` → Hugo default dirs |
| URLs     | `[permalinks]` — `posts`                                                                                                     |
| Feeds    | `[outputs]` — `home` / `section` include `RSS`                                                                               |
| Markdown | `[markup.goldmark]`, `[markup.highlight]`, `[markup.tableOfContents]`                                                        |
| Params   | `[params]` — defaults for SEO / OG                                                                                           |

---

## Deployment

Ship **`dist/`** as static files to any host (S3, nginx, CDN). No server runtime required.

---

## Environment

| Variable   | Description             | Default |
| ---------- | ----------------------- | ------- |
| `WEB_PORT` | Dev / preview bind port | `3001`  |

---

## References

### Hugo

- [Introduction](https://gohugo.io/about/introduction/)
- [Documentation](https://gohugo.io/documentation/)
- [Themes directory](https://themes.gohugo.io/)
- [Community forum](https://discourse.gohugo.io/)
- [Tailwind from Hugo (official)](https://gohugo.io/functions/css/tailwindcss/#setup)

### Layout / theme inspiration (not vendored)

- [hugo-book](https://github.com/alex-shpak/hugo-book/) — sidebar + TOC patterns
- [hugo-xmin](https://github.com/yihui/hugo-xmin) — minimal file layout
- [hugo-theme-nostyleplease](https://github.com/hanwenguo/hugo-theme-nostyleplease) — minimal HTML, bring-your-own CSS
- [PaperMod](https://github.com/adityatelange/hugo-PaperMod/) — SEO / head tag ideas
- [Hugoplate](https://github.com/zeon-studio/hugoplate) — feature-rich starter (reference only)

---

## Footer

Template © Luna. Hugo © [gohugoio](https://github.com/gohugoio/hugo).
