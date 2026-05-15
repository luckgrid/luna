# `apps/web` agent guide

Companion to [`README.md`](README.md). Read this **before** editing layouts or archetypes.

## Dispatcher pattern (locked — do not regress)

Four root templates live directly under `src/layouts/`:

```text
src/layouts/
├── baseof.html   # chrome — charset/viewport + partial head.html; body data hooks; site footer via site-footer.html
├── home.html     # kind=home  → same `<main>` as `page.html` simple branch (hero + article)
├── page.html     # kind=page  → `params.layout` branches (simple / article / collection) inlined
├── section.html  # kind=section → `params.layout` branches (list / catalog / collection) inlined
└── all.html      # ultimate fallback (taxonomy/term + safety net)
```

`page.html` and `section.html` normalize **`params.layout`** against an allowlist, then **`if` / `else if`** on `$layout` to render markup (each branch composes flat **`_partials/*.html`** fragments). Unknown values reset to **`simple`** / **`list`** (**D12**).

## Partials folder map

```text
src/layouts/_partials/
├── head.html    # <head>: meta/OG + deferred fingerprinted bundle.css
└── *.html       # flat fragments (site-header, hero, title-block, toc, …)
```

Calling conventions worth knowing:

- Document head: `partial "head.html" .` from [`baseof.html`](src/layouts/baseof.html) (meta tags + deferred CSS).
- Site chrome: `partial "site-header.html" .`, `partial "site-footer.html" .` (footer is wired from [`baseof.html`](src/layouts/baseof.html)).
- TOC: `partial "toc.html" .`
- Pagination: `partial "pagination.html" $paginator` (renders nothing when `TotalPages <= 1`).
- Hero: `partial "hero.html" .` or `partial "hero.html" (dict "page" . "slot" $html)` (catalog injects search form into the hero slot).
- Marketing strips from Markdown: shortcode [`latest-posts.html`](src/layouts/_shortcodes/latest-posts.html) (e.g. on the home page).

## Layout rules

1. **Do not** create `layouts/_default/`, `layouts/article/`, `layouts/catalog/`, or `layouts/collection/`. Hugo's type-folder lookup beats root templates and bypasses the dispatcher.
2. **Do not** add a `single.html` — `page.html` already handles kind=page.
3. **Page/section layout markup** lives in root **`page.html`** / **`section.html`** (and **`home.html`** for kind=home) — compose flat **`_partials/*.html`** fragments; do not add a `_partials/shell/` or `_partials/ui/` layer.
4. To add a new **`params.layout`** value, add a branch in the right root template + set `params.layout: <name>` in the matching archetype / cascade. Update the [README dispatcher table](README.md#layouts-dispatcher-pattern).
5. Do not reorder the locked collection DOM (D4): outer `<aside>` → `<main>` (page header + `<article>` + optional inner `<aside>` for TOC). CSS in `@luna/ds` positions the sidebars.
6. Marketing copy and optional strips (e.g. latest posts via **`latest-posts`**) live in **`content/*.md`**, rendered through the same **`page.html`** shapes — not hard-coded in `home.html`.

## Valid `params.layout` matrix (D11)

| Kind               | Allowed values                                                        | Default  |
| ------------------ | --------------------------------------------------------------------- | -------- |
| `page`             | `simple` (default), `article`, `collection`                           | `simple` |
| `section`          | `list` (default), `catalog`, `collection`                             | `list`   |
| `home`             | n/a — `home.html` inlines the same `<main>` as **`simple`** pages     | n/a      |
| `taxonomy`, `term` | n/a — handled by `all.html` (or future `taxonomy.html` / `term.html`) | n/a      |

Unknown `params.layout` values fall back to `simple` / `list` via the **D12** allowlist — the build won't break, but the rendered design won't match author intent; fix the front matter.

## Editing checklist (run before committing layout/archetype changes)

1. `cd apps/web && hugo --gc --minify` exits 0; `--templateMetrics` should show **`page.html`** / **`section.html`** / **`home.html`** + **`head.html`** and other **`_partials/*.html`** hits for the expected URLs.
2. Smoke-test the URL matrix:
   - `/` (home) → `home.html` (simple `<main>`) + `latest-posts` shortcode in content
   - `/posts/` → `section.html` (`catalog` branch)
   - `/posts/<slug>/` → `page.html` (`article` branch)
   - `/legal/` and `/posts/list-example/` → `section.html` (`collection` branch)
   - `/legal/<policy>/` and `/posts/list-example/<note>/` → `page.html` (`collection` branch)
   - `/tags/` and `/tags/<term>/` → `all.html`
3. Ripgrep for stale paths and stop on any non-zero result (only D-decision text in `README.md` / `AGENTS.md` should match):

   ```sh
   rg 'layouts/(catalog|article|collection)/' apps/web
   rg 'partial "(catalog|article/page|collection/section|collection/page)' apps/web/src/layouts
   ```

4. Prefer small **`_partials/*.html`** fragments for repeated markup; keep **`page.html`** / **`section.html`** readable. If a branch grows large, extract another flat partial — not a nested category folder unless Hugo requires it (e.g. `_markup/`).

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
