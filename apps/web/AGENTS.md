# `apps/web` agent guide

Companion to [`README.md`](README.md). Read this **before** editing layouts or archetypes.

## Dispatcher pattern (locked — do not regress)

Four root templates live directly under `src/layouts/`:

```text
src/layouts/
├── baseof.html   # chrome — body data hooks from .Params.layout / .Params.pattern
├── home.html     # kind=home  → calls _partials/page/simple.html (+ optional featured)
├── page.html     # kind=page  → dispatcher (≤ 5 lines)
├── section.html  # kind=section → dispatcher (≤ 5 lines)
└── all.html      # ultimate fallback (taxonomy/term + safety net)
```

`page.html` and `section.html` are **tiny dispatchers**:

```go-html-template
{{ define "main" }}
  {{- $layout := .Params.layout | default "simple" -}}
  {{- if not (templates.Exists (printf "_partials/page/%s.html" $layout)) }}{{ $layout = "simple" }}{{ end -}}
  {{- partial (printf "page/%s.html" $layout) . -}}
{{ end }}
```

`section.html` is the same shape with `"list"` as the default and `_partials/section/...` as the path. The `templates.Exists` check is the **D12 fallback guard** — unknown values render the safe default instead of erroring.

## Partials folder map

Behavior partials are grouped by concern, not by kind:

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

Calling conventions worth knowing:

- Every site chrome reference goes through `partial "layout/<name>.html" .`.
- The TOC partial is `partial "article/toc.html" .` (article concern, not page-kind concern).
- The pagination partial is `partial "list/pagination.html" $paginator` and only emits when `$paginator.TotalPages > 1`.
- `partial "hero.html" .` keeps the simple-page behavior; `partial "hero.html" (dict "page" . "slot" $rendered)` injects extra HTML inside the `<header data-hero>` (the catalog uses this for its search form).

## Layout rules

1. **Do not** create `layouts/_default/`, `layouts/article/`, `layouts/catalog/`, or `layouts/collection/`. Hugo's type-folder lookup beats root templates and bypasses the dispatcher.
2. **Do not** add a `single.html` — `page.html` already handles kind=page.
3. **Behavior** lives in partials under `_partials/page/<name>.html` (page kinds) or `_partials/section/<name>.html` (section kinds). One partial = one design.
4. To add a new design, add **one partial** + set `params.layout: <name>` in the matching archetype / cascade. Update the [README dispatcher table](README.md#layouts-dispatcher-pattern).
5. Do not reorder the locked collection DOM (D4): outer `<aside>` → `<main>` (page header + `<article>` + optional inner `<aside>` for TOC). CSS in `@luna/ds` positions the sidebars.
6. `home.html` stays ≤ 10 lines. Marketing copy goes in `content/_index.md` (rendered via `_partials/page/simple.html`), not hard-coded HTML.

## Valid `params.layout` matrix (D11)

| Kind               | Allowed values                                                        | Default  |
| ------------------ | --------------------------------------------------------------------- | -------- |
| `page`             | `simple` (default), `article`, `collection`                           | `simple` |
| `section`          | `list` (default), `catalog`, `collection`                             | `list`   |
| `home`             | n/a — `home.html` calls `_partials/page/simple.html` directly         | n/a      |
| `taxonomy`, `term` | n/a — handled by `all.html` (or future `taxonomy.html` / `term.html`) | n/a      |

Unknown `params.layout` values fall back to `simple` / `list` via the D12 guard. The build won't break, but the rendered design won't match author intent — fix the front matter.

## Decisions (D1–D12)

| ID      | Decision                                                                                                                                                                |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **D1**  | Drop `layouts/article/`. Root `page.html` dispatches to `_partials/page/<layout>.html`.                                                                                 |
| **D2**  | Drop `layouts/catalog/`. Root `section.html` dispatches to `_partials/section/<layout>.html`.                                                                           |
| **D3**  | Site-wide `[pagination] pagerSize = 10` in [`hugo.toml`](hugo.toml); templates call `.Paginate $coll` with no second arg.                                               |
| **D4**  | Collection DOM (hub + child) is locked: outer `<aside>` → `<main>` (page header + `<article>` + optional inner `<aside>` for TOC).                                      |
| **D5**  | Drop `layouts/collection/`. Collection lives in `_partials/page/collection.html` (child) + `_partials/section/collection.html` (hub).                                   |
| **D6**  | Catalog renders two collections: paginated list = `.Pages.ByDate.Reverse`; search index = `.RegularPagesRecursive ∪ (where .Pages "Kind" "section")`.                   |
| **D7**  | Keep root `home.html` as a thin delegator to `_partials/page/simple.html` + `_partials/article/featured.html`.                                                          |
| **D8**  | `all.html` is the ultimate fallback (catches taxonomy/term until/unless dedicated templates ship).                                                                      |
| **D9**  | Keep 4 archetypes (`default`, `article`, `catalog`, `collection`) selected by `-k`. Names match `params.layout` values.                                                 |
| **D10** | Conservative archetype defaults: no `category` / `weight` in `article.md`; `description: ""` in `default.md` + `catalog.md`; `params.pattern: catalog` in `catalog.md`. |
| **D11** | Single source of truth for valid `params.layout` (matrix above).                                                                                                        |
| **D12** | Dispatcher fallback guard via `templates.Exists`.                                                                                                                       |

## Editing checklist (run before committing layout/archetype changes)

1. `cd apps/web && hugo --gc --minify` exits 0 and shows `_partials/page/<layout>.html` + `_partials/section/<layout>.html` hits in `--templateMetrics` for the expected URLs.
2. Smoke-test the URL matrix:
   - `/` (home) → `home.html` → `_partials/page/simple.html` + featured
   - `/posts/` → `section.html` → `_partials/section/catalog.html`
   - `/posts/<slug>/` → `page.html` → `_partials/page/article.html`
   - `/legal/` and `/posts/list-example/` → `section.html` → `_partials/section/collection.html`
   - `/legal/<policy>/` and `/posts/list-example/<note>/` → `page.html` → `_partials/page/collection.html`
   - `/tags/` and `/tags/<term>/` → `all.html`
3. Ripgrep for stale paths and stop on any non-zero result (only D-decision text in `README.md` / `AGENTS.md` should match):

   ```sh
   rg 'layouts/(catalog|article|collection)/' apps/web
   rg 'partial "(catalog|article/page|collection/section|collection/page)' apps/web/src/layouts
   ```

4. If a template grows logic beyond a 5-line dispatcher, push the logic into a partial under `_partials/page/` or `_partials/section/`.

## Markdown render hooks (`_markup/`)

Custom Goldmark rendering lives in [`src/layouts/_markup/`](src/layouts/_markup/) ([Hugo render hooks](https://gohugo.io/render-hooks/)). All five hooks are wired and verified (see [README — Render hooks](README.md#render-hooks)):

- [`render-link.html`](src/layouts/_markup/render-link.html) — external links get `rel="noopener noreferrer" target="_blank"`.
- [`render-image.html`](src/layouts/_markup/render-image.html) — `loading="lazy"` + `decoding="async"`; standalone images become `<figure><figcaption>`. Requires `[markup.goldmark.parser] wrapStandAloneImageWithinParagraph = false` (set in [`hugo.toml`](hugo.toml)).
- [`render-heading.html`](src/layouts/_markup/render-heading.html) — `h2`–`h6` get `<a data-anchor href="#id">`; `h1` is left plain.
- [`render-blockquote.html`](src/layouts/_markup/render-blockquote.html) — GitHub-flavored alerts (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`) render as `<aside data-alert="<type>">`, matching [`_shortcodes/alert.html`](src/layouts/_shortcodes/alert.html). Plain `>` quotes stay `<blockquote>`.
- [`render-codeblock.html`](src/layouts/_markup/render-codeblock.html) — generic fallback. Wraps Chroma output in `<figure data-codeblock><figcaption>` when the fence carries `{filename="..."}`. Requires `[markup.goldmark.parser.attribute] block = true` in [`hugo.toml`](hugo.toml).
- [`render-codeblock-mermaid.html`](src/layouts/_markup/render-codeblock-mermaid.html) — ` ```mermaid ` → `<pre class="mermaid">`; Mermaid ESM lib loaded once per page via `.Page.Store`.
- [`render-codeblock-katex.html`](src/layouts/_markup/render-codeblock-katex.html) — ` ```katex ` → `<span class="katex-block">\[ … \]</span>`; KaTeX CSS + auto-render JS loaded once per page via `.Page.Store`.

**Adding a hook:** drop `_markup/render-<element>.html` (or `_markup/render-codeblock-<lang>.html`). Language-specific code-block hooks take precedence over the generic `render-codeblock.html`. Section-scoped overrides go at `src/layouts/<section>/_markup/<hook>.html` ([new template system folder map](https://gohugo.io/templates/new-templatesystem-overview/#example-folder-structure)).

**Other code fences** (`sh`, `js`, `go`, …) flow through `render-codeblock.html`, which delegates to Chroma. Add `{filename="..."}` to a fence to surface a filename caption above the highlighted block.

## Known follow-ups (deferred)

- New shortcodes (`details`, `columns`, `card`) are deferred — add only when a content page needs them.

## References

- [Hugo template types](https://gohugo.io/templates/types/)
- [Hugo template lookup order](https://gohugo.io/templates/lookup-order/)
- [Hugo new template system overview](https://gohugo.io/templates/new-templatesystem-overview/)
- Full blueprints (kept while refactor is fresh): [`temp/hugo-web-refactor-blueprints/README.md`](../../temp/hugo-web-refactor-blueprints/README.md)
