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
- **Collection sections** — [`legal/`](src/content/legal/) and [`posts/list-example/`](src/content/posts/list-example/) share [`src/layouts/_partials/collection-sidebar.html`](src/layouts/_partials/collection-sidebar.html): filter-as-you-type sidebar listing every page in the section, optional right **TOC** via [`TableOfContents`](https://gohugo.io/methods/page/tableofcontents/) when **`toc: true`** on the page. Collection routes carry **`data-layout="collection"`** and a section-specific **`data-pattern`** on `<body>` (see [`main.css`](src/assets/css/main.css)).
- **[Archetypes](https://gohugo.io/content-management/archetypes/)** — four starters (**`default`**, **`catalog`**, **`article`**, **`collection`**) with **`-k`**; see [Archetypes](#archetypes-hugo-new) below.
- **Dispatcher layouts** — [`baseof.html`](src/layouts/baseof.html) plus [`home.html`](src/layouts/home.html), [`page.html`](src/layouts/page.html), [`section.html`](src/layouts/section.html), [`all.html`](src/layouts/all.html). **`page.html`** and **`section.html`** branch on **`params.layout`** inline (each branch composes flat **`_partials/*.html`** fragments). See [Layouts (dispatcher pattern)](#layouts-dispatcher-pattern).
- **SEO**: meta description, canonical URL, minimal Open Graph / Twitter tags in [`src/layouts/_partials/metadata.html`](src/layouts/_partials/metadata.html) (with deferred CSS link); [sitemap](https://gohugo.io/templates/sitemap-template/), [RSS](https://gohugo.io/templates/rss/).
- **Shortcodes** — e.g. [`alert.html`](src/layouts/_shortcodes/alert.html), [`latest-posts.html`](src/layouts/_shortcodes/latest-posts.html).
- **Markdown render hooks** — [`src/layouts/_markup/`](src/layouts/_markup/) overrides Goldmark's default rendering for links, images, headings, blockquotes, and code blocks (generic Chroma + Mermaid + KaTeX) ([Hugo render hooks](https://gohugo.io/render-hooks/)). See [Render hooks](#render-hooks) below.
- **Taxonomies enabled** — default `tags` and `categories` resolve to `/tags/` and `/categories/`, rendered by the catch-all [`all.html`](src/layouts/all.html) until a dedicated `taxonomy.html` / `term.html` ships ([`hugo.toml`](hugo.toml) sets `disableKinds = []`).

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

Layouts follow Hugo’s **[new template system](https://gohugo.io/templates/new-templatesystem-overview/)** (v0.146+) and use a **dispatcher pattern** (see [Layouts (dispatcher pattern)](#layouts-dispatcher-pattern) and [`AGENTS.md`](AGENTS.md)). Root [`baseof.html`](src/layouts/baseof.html) is document chrome only; [`home.html`](src/layouts/home.html) handles kind=home with the same `<main>` as **`simple`** pages; root [`page.html`](src/layouts/page.html) and [`section.html`](src/layouts/section.html) branch on **`params.layout`** inline; [`all.html`](src/layouts/all.html) is the ultimate fallback (taxonomy/term + safety net). Body hooks come from front matter **`params.layout`** / **`params.pattern`**. Shared fragments live as flat files under **`layouts/_partials/`** (e.g. `hero.html`, `header.html`, `article-meta.html` for article meta); shortcodes in **`layouts/_shortcodes/`**. References: [Hugo template types](https://gohugo.io/templates/types/), [template lookup order](https://gohugo.io/templates/lookup-order/), [introduction](https://gohugo.io/templates/introduction/).

```text
src/layouts/_partials/
├── metadata.html              # meta/OG tags
├── css.html                   # deferred fingerprinted stylesheet link
├── header.html, footer.html, navigation.html, brand.html
├── hero.html, article-header.html, breadcrumbs.html, article-meta.html, article-toc.html
├── article-card.html, search-filter.html, pagination.html
└── collection-sidebar.html, article-footer.html
```

Key pieces:

- [`metadata.html`](src/layouts/_partials/metadata.html): `<title>`, description, canonical, Open Graph / Twitter
- [`css.html`](src/layouts/_partials/css.html): deferred [`bundle.css`](src/assets/css/bundle.css) link (fingerprinted outside dev; wired from [`baseof.html`](src/layouts/baseof.html))
- [`header.html`](src/layouts/_partials/header.html) / [`footer.html`](src/layouts/_partials/footer.html): brand + primary nav; footer nav (footer is included from [`baseof.html`](src/layouts/baseof.html))
- [`navigation.html`](src/layouts/_partials/navigation.html): link list for primary + footer navigation; accepts `{ page }` (wrapped in `<nav>` by `header.html` / `footer.html`)
- [`hero.html`](src/layouts/_partials/hero.html): `<header data-hero>` — page context or `(dict "page" . "slot" $html)` (catalog injects search form)
- [`article-header.html`](src/layouts/_partials/article-header.html): `<hgroup>` (category, title, description, optional [`article-meta.html`](src/layouts/_partials/article-meta.html) for `kind=page`)
- [`breadcrumbs.html`](src/layouts/_partials/breadcrumbs.html): logo link + breadcrumbs (standalone articles)
- [`collection-sidebar.html`](src/layouts/_partials/collection-sidebar.html): collection nav + filter search
- [`article-toc.html`](src/layouts/_partials/article-toc.html): optional right-hand “On this page” TOC (renders nothing when disabled or empty)
- [`article-card.html`](src/layouts/_partials/article-card.html): card link for list/catalog/home/collection shortcode
- [`article-footer.html`](src/layouts/_partials/article-footer.html): collection prev/next nav inside article pages
- [`search-filter.html`](src/layouts/_partials/search-filter.html): `<form role="search">` with search input + category `<select>`; accepts `{ id, label, placeholder?, categories?, … }`
- [`pagination.html`](src/layouts/_partials/pagination.html): paginator nav; accepts `.Paginate` output

**Scaling:** add another **collection** subtree under `content/` by creating its `_index.md` with **`type: collection`** + **`params.layout: collection`** and a **`cascade`** block for child articles. Add another searchable article index with **`params.layout: catalog`**. Add a new design by extending the **`if` / `else if`** branches in [`page.html`](src/layouts/page.html) or [`section.html`](src/layouts/section.html) plus **`params.layout: <name>`** in front matter or cascade; add shared markup as a new partial next to the existing flat **`_partials/*.html`** files.

### Layouts (dispatcher pattern)

| URL                                       | Root template                                                                                    | Where markup lives                                                                             |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `/`                                       | [`home.html`](src/layouts/home.html)                                                             | Inlined simple `<main>` + **`latest-posts`** shortcode in [`_index.md`](src/content/_index.md) |
| `/about/` (or any simple page)            | [`page.html`](src/layouts/page.html) (`params.layout=simple` default)                            | **`simple`** branch in [`page.html`](src/layouts/page.html)                                    |
| `/posts/`                                 | [`section.catalog.html`](src/layouts/section.catalog.html) (`params.layout=catalog`)             | Inlined in [`section.catalog.html`](src/layouts/section.catalog.html)                          |
| `/posts/<slug>/`                          | [`page.html`](src/layouts/page.html) (`params.layout=article` cascaded)                          | **`article`** branch in [`page.html`](src/layouts/page.html)                                   |
| `/legal/`                                 | [`section.html`](src/layouts/section.html) (`params.layout=collection`)                          | **`collection`** branch in [`section.html`](src/layouts/section.html)                          |
| `/legal/<policy>/`                        | [`page.html`](src/layouts/page.html) (`params.layout=collection` cascaded)                       | **`collection`** branch in [`page.html`](src/layouts/page.html)                                |
| `/posts/list-example/`                    | [`section.html`](src/layouts/section.html) (`params.layout=collection`, overrides posts cascade) | **`collection`** branch in [`section.html`](src/layouts/section.html)                          |
| `/posts/list-example/<note>/`             | [`page.html`](src/layouts/page.html) (`params.layout=collection` cascaded)                       | **`collection`** branch in [`page.html`](src/layouts/page.html)                                |
| `/tags/`, `/tags/<term>/`, `/categories/` | [`all.html`](src/layouts/all.html) (taxonomy/term fallback)                                      | (inline list)                                                                                  |

Both root templates normalize **`params.layout`** against an allowlist so an unknown value falls back to the safe default (`simple` for pages, `list` for sections) instead of erroring.

#### Valid `params.layout` matrix

| Kind               | Allowed                                                               | Default  |
| ------------------ | --------------------------------------------------------------------- | -------- |
| `page`             | `simple` (default), `article`, `collection`                           | `simple` |
| `section`          | `list` (default), `catalog`, `collection`                             | `list`   |
| `home`             | n/a — `home.html` inlines the same `<main>` as **`simple`** pages     | n/a      |
| `taxonomy`, `term` | n/a — handled by `all.html` (or future `taxonomy.html` / `term.html`) | n/a      |

#### Decisions (D1–D12)

| ID      | Summary                                                                                                                                                                                                                                                        |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **D1**  | Drop `layouts/article/`. Root [`page.html`](src/layouts/page.html) holds all **`kind=page`** layout branches.                                                                                                                                                  |
| **D2**  | Drop `layouts/catalog/`. Root [`section.html`](src/layouts/section.html) holds all **`kind=section`** layout branches.                                                                                                                                         |
| **D3**  | Site-wide [`[pagination] pagerSize = 10`](hugo.toml); templates call `.Paginate $coll` with no second arg.                                                                                                                                                     |
| **D4**  | Collection DOM (hub + child): outer `<aside>` (collection nav) → `<main>` containing page header + `<article>` + optional inner `<aside>` (TOC).                                                                                                               |
| **D5**  | Drop `layouts/collection/`. Collection markup lives in the **`collection`** branches of [`page.html`](src/layouts/page.html) (child) and [`section.html`](src/layouts/section.html) (hub).                                                                     |
| **D6**  | Catalog (`/posts/`) renders two collections: paginated list = `.Pages.ByDate.Reverse` (direct children + collection landings, no grandchildren); search index = `.RegularPagesRecursive ∪ (where .Pages "Kind" "section")` (every leaf + collection landings). |
| **D7**  | Root `home.html` mirrors the **`simple`** branch of [`page.html`](src/layouts/page.html). Latest posts use **`latest-posts`** in `content/_index.md` (same pool: `/posts/` children with `Kind` `page`).                                                       |
| **D8**  | `all.html` is the ultimate fallback (catches taxonomy/term until/unless dedicated templates ship).                                                                                                                                                             |
| **D9**  | Keep 4 archetypes selected by `-k`: `default`, `article`, `catalog`, `collection`. Names match `params.layout` values.                                                                                                                                         |
| **D10** | Conservative archetype defaults: no `category` / `weight` in `article.md`; `description: ""` in `default.md` + `catalog.md`; `params.pattern: catalog` in `catalog.md`.                                                                                        |
| **D11** | Single source of truth for valid `params.layout` (matrix above).                                                                                                                                                                                               |
| **D12** | Normalize **`params.layout`** to an allowlist; unknown values fall back to **`simple`** / **`list`**.                                                                                                                                                          |

Full design rationale + acceptance criteria live in `temp/hugo-web-refactor-blueprints/` (kept while the refactor is fresh; remove after the next release cycle).

### Render hooks

Custom Markdown rendering lives in [`src/layouts/_markup/`](src/layouts/_markup/) ([Hugo render hooks](https://gohugo.io/render-hooks/), [new template system file map](https://gohugo.io/templates/new-templatesystem-overview/#example-folder-structure)). Each hook intercepts one Goldmark element after parse and before HTML output:

| Hook file                                                                            | What it does                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`render-link.html`](src/layouts/_markup/render-link.html)                           | External links (different scheme + host than `baseURL`) get `rel="noopener noreferrer" target="_blank"`. Internal, anchor (`#…`), `mailto:`, and `tel:` links pass through unchanged.                                                                                                                                                                                                                                                                                                                                                                   |
| [`render-image.html`](src/layouts/_markup/render-image.html)                         | All images get `loading="lazy"` + `decoding="async"`. Standalone images (`.IsBlock`) are wrapped in `<figure>` with optional `<figcaption>` from the image title; inline images stay inline. Requires `[markup.goldmark.parser] wrapStandAloneImageWithinParagraph = false`.                                                                                                                                                                                                                                                                            |
| [`render-heading.html`](src/layouts/_markup/render-heading.html)                     | `h2`–`h6` get a trailing `<a data-anchor>` self-link for deep-linking and TOC alignment; `h1` stays plain (page-header / hero owns it).                                                                                                                                                                                                                                                                                                                                                                                                                 |
| [`render-blockquote.html`](src/layouts/_markup/render-blockquote.html)               | Regular `>` quotes pass through as `<blockquote>`. GitHub-flavored alerts (`> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, `> [!CAUTION]`) render as `<aside role="note" data-alert="<type>">` to match the existing [`_shortcodes/alert.html`](src/layouts/_shortcodes/alert.html) `data-alert` contract in `@luna/ds`.                                                                                                                                                                                                                     |
| [`render-codeblock.html`](src/layouts/_markup/render-codeblock.html)                 | Generic fallback for every fenced code block. Hands content to Hugo's [`transform.HighlightCodeBlock`](https://gohugo.io/functions/transform/highlightcodeblock/) (Chroma) and, when the fence carries `{filename="…"}`, wraps the highlighted output in `<figure data-codeblock><figcaption><code>…</code></figcaption>…</figure>`. Requires `[markup.goldmark.parser.attribute] block = true`. Theme reference: [hugo-book `_markup/render-codeblock.html`](https://github.com/alex-shpak/hugo-book/blob/main/layouts/_markup/render-codeblock.html). |
| [`render-codeblock-mermaid.html`](src/layouts/_markup/render-codeblock-mermaid.html) | ` ```mermaid ` fences emit `<pre class="mermaid">`; the Mermaid ESM lib is loaded once per page via `.Page.Store` so pages without diagrams don't pay the cost. Theme reference: [hugo-book `_markup/render-codeblock-mermaid.html`](https://github.com/alex-shpak/hugo-book/blob/main/layouts/_markup/render-codeblock-mermaid.html).                                                                                                                                                                                                                  |
| [`render-codeblock-katex.html`](src/layouts/_markup/render-codeblock-katex.html)     | ` ```katex ` fences emit `<span class="katex-block">\[ … \]</span>` and lazily load KaTeX CSS + auto-render JS once per page via `.Page.Store`. Theme reference: [hugo-book `_markup/render-codeblock-katex.html`](https://github.com/alex-shpak/hugo-book/blob/main/layouts/_markup/render-codeblock-katex.html).                                                                                                                                                                                                                                      |

Other code fences (`sh`, `js`, `go`, …) flow through `render-codeblock.html`, which delegates to Hugo's built-in [Chroma](https://github.com/alecthomas/chroma) highlighter (configured under `[markup.highlight]`). Add `{filename="run.sh"}` to a fence to surface a filename caption above the highlighted block. Language-specific code-block hooks (`render-codeblock-mermaid.html`, `render-codeblock-katex.html`) take precedence over the generic fallback.

To add a new hook, drop a file under `_markup/` (e.g. `render-codeblock-<lang>.html`). To override only inside a specific section, place the hook at `src/layouts/<section>/_markup/<hook>.html` per the [new template system folder structure](https://gohugo.io/templates/new-templatesystem-overview/#example-folder-structure).

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

Production CSS is loaded from **`src/assets/css/bundle.css`** and linked from [`src/layouts/_partials/css.html`](src/layouts/_partials/css.html) (deferred fingerprint in production). If that file is missing, Hugo warns — run the Tailwind step first (Moon always runs it before `go tool hugo`).

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
- **`collection`** -> [`collection.md`](src/archetypes/collection.md): **collection** `_index` **frontmatter** (including **`cascade`** sidebar labels); add intro and body in the scaffolded file under **`src/content/`**

Tune **`cascade`** / **`toc`** / **`weight`** on real hubs after scaffolding. **`collection.md`** includes a starter **`cascade.params`** block—replace labels per section.

| Role                      | Path                                                                 | Output                                       |
| ------------------------- | -------------------------------------------------------------------- | -------------------------------------------- |
| Home                      | [`src/content/_index.md`](src/content/_index.md)                     | `/`                                          |
| Posts index               | [`src/content/posts/_index.md`](src/content/posts/_index.md)         | `/posts/`                                    |
| Post                      | `src/content/posts/<slug>.md`                                        | `/posts/<slug>/`                             |
| List example (collection) | [`src/content/posts/list-example/`](src/content/posts/list-example/) | `/posts/list-example/` (hub + nested posts)  |
| Legal (collection)        | [`src/content/legal/`](src/content/legal/)                           | `/legal/` (hub), `/legal/<slug>/` (policies) |

### Collection layout (article hubs: legal & list-example)

**`legal`** and **`posts/list-example`** are **collections**: same sidebar partial ([`collection-sidebar.html`](src/layouts/_partials/collection-sidebar.html)) with labels driven by **`cascade`** `params` on each section’s `_index.md` (`collection_sidebar_search_label`, `collection_nav_aria`). The **collection hub** uses the **`collection`** branch in [`section.html`](src/layouts/section.html); child pages use the **`collection`** branch in [`page.html`](src/layouts/page.html). Both branches follow the locked outer DOM (D4): outer `<aside>` (sidebar nav) → `<main>` with page header + `<article>` + optional inner `<aside>` (TOC) when **`toc: true`** (renders as **`aside[data-toc]`**).

- **Legal hub** — [`src/content/legal/_index.md`](src/content/legal/_index.md); policies as `src/content/legal/<slug>.md` with **`weight`** for sidebar order.
- **List-example hub** — [`src/content/posts/list-example/_index.md`](src/content/posts/list-example/_index.md); new hubs: **`hugo new content …/_index.md -k collection`**. Pages inside the hub: **`hugo new content posts/list-example/<slug>.md -k article`** ([`article.md`](src/archetypes/article.md)).

Search matches **title**, **description**, and URL path segments (client-side; no external search service).

### Front matter (common)

Common front matter:

- `title`: page heading / `<title>`
- `description`: meta description
- `date`: post date (`YYYY-MM-DD`)
- `category`: optional display metadata for posts (no longer in archetypes per D10 — add per file when needed)
- `tags`: drives `/tags/<term>/` (taxonomy enabled — see D8)
- `slug`: optional URL slug override
- `weight`: sidebar order in **collection** sections (`legal`, `posts/list-example`, ...)
- `toc`: if true, render right-hand **`aside[data-toc]`** in **collection** and **article** templates
- `params.layout`: **dispatcher key** for `page.html` / `section.html` (see [matrix](#valid-paramslayout-matrix)); also passed to `<body>` as `data-layout`
- `params.pattern`: passed to `<body>` as `data-pattern` for **[@luna/ds article layout](../../packages/ds/src/layouts/article.css)** styling
- `type`: Hugo native page type — **metadata only** post-D1/D2 (no template lookup depends on it; kept for future RSS-per-type or content filtering)

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
