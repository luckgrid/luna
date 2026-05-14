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
- **Dispatcher layouts** — four root templates ([`home.html`](src/layouts/home.html), [`page.html`](src/layouts/page.html), [`section.html`](src/layouts/section.html), [`all.html`](src/layouts/all.html)) plus [`baseof.html`](src/layouts/baseof.html). `page.html` and `section.html` are 3-line dispatchers that delegate to behavior partials under [`_partials/page/`](src/layouts/_partials/page/) and [`_partials/section/`](src/layouts/_partials/section/) keyed on **`params.layout`**. See [Layouts (dispatcher pattern)](#layouts-dispatcher-pattern).
- **SEO**: meta description, canonical URL, minimal Open Graph / Twitter tags grouped in [`src/layouts/_partials/head/metadata.html`](src/layouts/_partials/head/metadata.html); [sitemap](https://gohugo.io/templates/sitemap-template/), [RSS](https://gohugo.io/templates/rss/).
- **Shortcodes** — e.g. [`src/layouts/_shortcodes/alert.html`](src/layouts/_shortcodes/alert.html) (`{{< alert >}}...{{< /alert >}}`).
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

Layouts follow Hugo’s **[new template system](https://gohugo.io/templates/new-templatesystem-overview/)** (v0.146+) and use a **dispatcher pattern** (see [Layouts (dispatcher pattern)](#layouts-dispatcher-pattern) and [`AGENTS.md`](AGENTS.md)). Root [`baseof.html`](src/layouts/baseof.html) is document chrome only; [`home.html`](src/layouts/home.html) handles kind=home; root [`page.html`](src/layouts/page.html) and [`section.html`](src/layouts/section.html) are tiny dispatchers that fan out to behavior partials under [`_partials/page/`](src/layouts/_partials/page/) and [`_partials/section/`](src/layouts/_partials/section/) keyed on **`params.layout`**; [`all.html`](src/layouts/all.html) is the ultimate fallback (taxonomy/term + safety net). Body hooks come from front matter **`params.layout`** / **`params.pattern`**. Partials live in **`layouts/_partials/`**; shortcodes in **`layouts/_shortcodes/`**. References: [Hugo template types](https://gohugo.io/templates/types/), [template lookup order](https://gohugo.io/templates/lookup-order/), [introduction](https://gohugo.io/templates/introduction/).

Partials are grouped by concern, not by kind:

```text
src/layouts/_partials/
├── layout/      # site chrome (header, nav, footer) — used by baseof + section partials
├── page/        # one file per page-kind design (simple, article, collection) + page/header.html
├── section/     # one file per section-kind design (list, catalog, collection)
├── article/     # article chrome (header w/ breadcrumbs, footer, metadata, card, featured, toc, collection)
├── list/        # reusable list scaffolding (cards, search-form, pagination)
├── head/        # <head> fragments (metadata, css)
├── brand.html   # site logo + name
└── hero.html    # `<header data-hero>` — accepts a page or { page, slot } dict
```

Key partials:

- [`layout/header.html`](src/layouts/_partials/layout/header.html): shared top header for **catalog** and **collection** pages (brand + nav)
- [`layout/nav.html`](src/layouts/_partials/layout/nav.html): primary + footer navigation; accepts `{ page, label }`
- [`layout/footer.html`](src/layouts/_partials/layout/footer.html): site footer, called from [`baseof.html`](src/layouts/baseof.html)
- [`hero.html`](src/layouts/_partials/hero.html): `<header data-hero>` for sections and simple pages. Accepts the page directly **or** `(dict "page" . "slot" $rendered)` so a caller can inject an extra HTML fragment (the catalog uses this to inject its search form)
- [`page/header.html`](src/layouts/_partials/page/header.html): page-level `<header>` with `<hgroup>` (category, title, description, optional date/tags metadata for `kind=page`)
- [`article/header.html`](src/layouts/_partials/article/header.html): article-page header with logo-only home link + breadcrumbs
- [`article/collection.html`](src/layouts/_partials/article/collection.html): collection navigation with filter-as-you-type search
- [`article/toc.html`](src/layouts/_partials/article/toc.html): reusable “On this page” aside for collection and article layouts
- [`article/card.html`](src/layouts/_partials/article/card.html): card link for a page or collection index in catalogs, collections, and featured-post sections
- [`article/featured.html`](src/layouts/_partials/article/featured.html): home **latest** list (`dict` with `title`, `pages`)
- [`list/cards.html`](src/layouts/_partials/list/cards.html): generic `<section data-list><ul>…cards…</ul></section>` wrapper; accepts `{ pages, aria, id? }`
- [`list/search-form.html`](src/layouts/_partials/list/search-form.html): reusable `<form role="search">`; accepts `{ id, label, placeholder?, name?, submit? }`
- [`list/pagination.html`](src/layouts/_partials/list/pagination.html): paginator nav (prev/next + page numbers); accepts a paginator (the value of `.Paginate`) and renders nothing when `TotalPages <= 1`

**Scaling:** add another **collection** subtree under `content/` by creating its `_index.md` with **`type: collection`** + **`params.layout: collection`** in `params`, and a **`cascade`** block for child articles. Add another searchable article index by creating a new section `_index.md` with **`params.layout: catalog`**. Add a new design by creating a new partial under `_partials/page/<name>.html` (or `_partials/section/<name>.html`) and setting **`params.layout: <name>`** in front matter or cascade.

### Layouts (dispatcher pattern)

| URL                                       | Root template                                                                                    | Behavior partial                                                                                                                                            |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/`                                       | [`home.html`](src/layouts/home.html)                                                             | [`_partials/page/simple.html`](src/layouts/_partials/page/simple.html) (+ [`_partials/article/featured.html`](src/layouts/_partials/article/featured.html)) |
| `/about/` (or any simple page)            | [`page.html`](src/layouts/page.html) (`params.layout=simple` default)                            | [`_partials/page/simple.html`](src/layouts/_partials/page/simple.html)                                                                                      |
| `/posts/`                                 | [`section.html`](src/layouts/section.html) (`params.layout=catalog`)                             | [`_partials/section/catalog.html`](src/layouts/_partials/section/catalog.html)                                                                              |
| `/posts/<slug>/`                          | [`page.html`](src/layouts/page.html) (`params.layout=article` cascaded)                          | [`_partials/page/article.html`](src/layouts/_partials/page/article.html)                                                                                    |
| `/legal/`                                 | [`section.html`](src/layouts/section.html) (`params.layout=collection`)                          | [`_partials/section/collection.html`](src/layouts/_partials/section/collection.html)                                                                        |
| `/legal/<policy>/`                        | [`page.html`](src/layouts/page.html) (`params.layout=collection` cascaded)                       | [`_partials/page/collection.html`](src/layouts/_partials/page/collection.html)                                                                              |
| `/posts/list-example/`                    | [`section.html`](src/layouts/section.html) (`params.layout=collection`, overrides posts cascade) | [`_partials/section/collection.html`](src/layouts/_partials/section/collection.html)                                                                        |
| `/posts/list-example/<note>/`             | [`page.html`](src/layouts/page.html) (`params.layout=collection` cascaded)                       | [`_partials/page/collection.html`](src/layouts/_partials/page/collection.html)                                                                              |
| `/tags/`, `/tags/<term>/`, `/categories/` | [`all.html`](src/layouts/all.html) (taxonomy/term fallback)                                      | (inline list)                                                                                                                                               |

Both dispatchers wrap the partial call in a **`templates.Exists` guard** so an unknown `params.layout` value falls back to the safe default (`simple` for pages, `list` for sections) instead of erroring.

#### Valid `params.layout` matrix

| Kind               | Allowed                                                               | Default  |
| ------------------ | --------------------------------------------------------------------- | -------- |
| `page`             | `simple` (default), `article`, `collection`                           | `simple` |
| `section`          | `list` (default), `catalog`, `collection`                             | `list`   |
| `home`             | n/a — `home.html` calls `_partials/page/simple.html` directly         | n/a      |
| `taxonomy`, `term` | n/a — handled by `all.html` (or future `taxonomy.html` / `term.html`) | n/a      |

#### Decisions (D1–D12)

| ID      | Summary                                                                                                                                                                                                                                                        |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **D1**  | Drop `layouts/article/`. Root `page.html` dispatches via `partial (printf "page/%s.html" (.Params.layout \| default "simple")) .`.                                                                                                                             |
| **D2**  | Drop `layouts/catalog/`. Root `section.html` dispatches via `partial (printf "section/%s.html" (.Params.layout \| default "list")) .`.                                                                                                                         |
| **D3**  | Site-wide [`[pagination] pagerSize = 10`](hugo.toml); templates call `.Paginate $coll` with no second arg.                                                                                                                                                     |
| **D4**  | Collection DOM (hub + child): outer `<aside>` (collection nav) → `<main>` containing page header + `<article>` + optional inner `<aside>` (TOC).                                                                                                               |
| **D5**  | Drop `layouts/collection/`. Collection behavior lives in `_partials/page/collection.html` (child) and `_partials/section/collection.html` (hub).                                                                                                               |
| **D6**  | Catalog (`/posts/`) renders two collections: paginated list = `.Pages.ByDate.Reverse` (direct children + collection landings, no grandchildren); search index = `.RegularPagesRecursive ∪ (where .Pages "Kind" "section")` (every leaf + collection landings). |
| **D7**  | Keep root `home.html` as a thin delegator to `_partials/page/simple.html` + optional `_partials/article/featured.html`.                                                                                                                                        |
| **D8**  | `all.html` is the ultimate fallback (catches taxonomy/term until/unless dedicated templates ship).                                                                                                                                                             |
| **D9**  | Keep 4 archetypes selected by `-k`: `default`, `article`, `catalog`, `collection`. Names match `params.layout` values.                                                                                                                                         |
| **D10** | Conservative archetype defaults: no `category` / `weight` in `article.md`; `description: ""` in `default.md` + `catalog.md`; `params.pattern: catalog` in `catalog.md`.                                                                                        |
| **D11** | Single source of truth for valid `params.layout` (matrix above).                                                                                                                                                                                               |
| **D12** | Dispatcher fallback guard via `templates.Exists` so unknown `params.layout` values render the safe default instead of erroring.                                                                                                                                |

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

**`legal`** and **`posts/list-example`** are **collections**: same navigation partial ([`article/collection.html`](src/layouts/_partials/article/collection.html)) with labels driven by **`cascade`** `params` on each section’s `_index.md` (`collection_sidebar_search_label`, `collection_nav_aria`). The **collection hub** is rendered by [`_partials/section/collection.html`](src/layouts/_partials/section/collection.html); child pages share the same shell via [`_partials/page/collection.html`](src/layouts/_partials/page/collection.html). Both partials follow the locked outer DOM (D4): outer `<aside>` (sidebar nav) → `<main>` with page header + `<article>` + optional inner `<aside>` (TOC) when **`toc: true`** (renders as **`aside[data-toc]`**).

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
