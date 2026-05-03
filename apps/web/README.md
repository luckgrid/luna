# `web`

Luna static marketing/content site, built with **Go + [templ](https://templ.guide/)**
and Markdown content via [goldmark](https://github.com/yuin/goldmark).

The interactive/SSR app lives at [`apps/app`](../app/README.md).

## What this app demonstrates

- `src/content/` is the **Markdown content source** (pages + catalog items).
- `src/pages/` is the **templ-only page layer** (layout/structure).

Together they act like a tiny file router (mirroring `apps/app`):

- `src/pages/index.templ` → `/` (home page)
- `src/content/<name>.md` → `/<name>.html` (single page from Markdown,
  rendered through a dedicated template registered in `main.go`)
- `src/pages/<catalog>/index.templ` + `src/pages/<catalog>/post.templ`
  and `src/content/<catalog>/*.md` → catalog (`/<catalog>/` index +
  `/<catalog>/<slug>/` per post)

The same `lib.ParseFile` / `lib.ParseDir` pipeline parses Markdown for
both top-level pages and catalog posts; only the templ component used to
render the HTML differs.

> Note: `apps/app` (SolidStart) uses the bracketed file-system router
> convention (`[id].tsx`). Go rejects square brackets in source
> filenames, so this app uses plain `index.templ` / `post.templ` — the
> dynamic-template intent is conveyed by colocation with the Markdown
> files instead.

## Layout

```text
apps/web/
  main.go                       Single entry + SSG runner (Build/Serve)
  src/
    components/                 Reusable templ chrome
      layout.templ              LayoutHeader, LayoutFooter
      page.templ                PageHeader, ListSection
    layouts/                    Two layouts that compose the components
      base.templ                @Base(title, description) — html shell
      post.templ                @Post(p) — article header + tags + body slot
    lib/                        SSG building blocks (no templ deps)
      post.go                   Post type, ParseFile, ParseDir, taxonomy helpers
      page.go                   WritePage
      fs.go                     CopyTree, ServeDir
    content/                    Markdown content (source of truth)
      legal.md                  → /legal.html
      posts/
        <slug>.md               → /posts/<slug>/
        ...                     (add more posts here)
    pages/                      Templ-only page modules (mirrors apps/app/src/routes)
      index.templ               /
      legal.templ               /legal.html renderer (registered in main.go)
      posts/                    Catalog module
        index.templ             /posts/  (catalog index, grouped by category)
        post.templ              /posts/<slug>/ (per-post detail)
    styles.css                  Entry CSS — imports @luna/ds + @source declarations
  public/                       Static assets copied verbatim into dist/
  dist/                         Generated output (gitignored)
```

This structure follows the [templ project structure
guide](https://templ.guide/project-structure/project-structure) (the
`counter` example), adapted for static-site generation with explicit
layout composition and a thin file-router under `pages/`.

## How layouts compose

Two layouts, both composing components from `src/components/`:

- **`layouts.Base(title, description)`** — emits `<html>`/`<head>`/
  `<body>` with `LayoutHeader` and `LayoutFooter`. Yields `{ children... }`
  inside `<main>`.
- **`layouts.Post(p)`** — wraps `Base` and adds the article header
  (`PageHeader` + date + tags). Yields `{ children... }` for the
  Markdown body.

Anything more elaborate is built in the page template itself by
composing `Base` with `PageHeader`, `ListSection`, and any custom
markup. For example, `src/pages/posts/index.templ`:

```go
templ Page(posts []lib.Post) {
    @layouts.Base("Posts", "Articles, announcements, and guides.") {
        @components.PageHeader("", "Posts", "Articles, announcements, and guides.")
        {{ grouped := lib.GroupByCategory(posts) }}
        {{ categories := lib.Categories(posts) }}
        for _, cat := range categories {
            @components.ListSection(cat, grouped[cat])
        }
    }
}
```

## Adding content

### A single Markdown page

1. Create `src/content/<name>.md` with frontmatter (`title`,
   `description`, optional `category`, `date`, `tags`).
2. (Optional) Create `src/pages/<name>.templ` for a dedicated renderer
   and register it in `main.go` via `Site.Pages`.
3. Rebuild — it routes to `/<name>.html` using either the dedicated
   template.

### A catalog post

1. Drop `src/content/posts/<slug>.md` (the filename is the slug). Set
   `category:` in frontmatter to group it on the index page; the
   category never appears in the URL.
2. Rebuild — it routes to `/posts/<slug>/`.

### A new catalog

1. Create `src/pages/<catalog>/index.templ` (Page func — index renderer)
   and `src/pages/<catalog>/post.templ` (Post func — detail renderer)
   in a new Go package.
2. Drop Markdown files at `src/content/<catalog>/<slug>.md`.
3. Register in [`main.go`](main.go):

   ```go
   site.Catalogs = append(site.Catalogs, Catalog{
       Name:   "<catalog>",
       Index:  <catalog>.Page,
       Detail: <catalog>.Post,
   })
   ```

## Frontmatter

| Key           | Purpose                                               |
| ------------- | ----------------------------------------------------- |
| `title`       | Page title (also list label and `<title>`)            |
| `description` | Meta description / list snippet / lede                |
| `date`        | YYYY-MM-DD — drives newest-first sort order           |
| `category`    | Catalog grouping label (not in URL)                   |
| `tags`        | List of strings — rendered in `layouts.Post`          |
| `slug`        | Optional override; defaults to filename without `.md` |

## Prerequisites

- Toolchains pinned in [`.prototools`](../../.prototools) (Go, Bun, etc.). From the repo root run `bun run install` or `proto install` so `go` resolves to that toolchain.
- The [`templ`](https://templ.guide/) CLI is pinned as a [Go tool](https://templ.guide/quick-start/installation#go-install-as-tool) in [`go.mod`](go.mod) (`tool github.com/a-h/templ/cmd/templ`). Moon runs `go tool templ` — no global `go install` step. To regenerate templates manually: `go tool templ generate` from this directory.

## Build and serve

From the repo root:

```sh
moon run web:build       # templ generate → tailwindcss CLI → SSG → dist/
moon run web:dev         # watch templ + serve dist/ on $WEB_PORT (default 3000)
```

`bun run dev` from the repo root boots `web`, `app`, and `api` together;
`web:dev` depends on `web:build`, so moon runs modules, templ, Tailwind, and a
full SSG pass before the watch server starts.

## Deployment

Deployment infrastructure (CDK / Pulumi / etc.) is intentionally not
included yet — `dist/` is a plain folder of HTML/CSS that any static
host can serve.

## Environment variables

| Variable   | Description     | Default |
| ---------- | --------------- | ------- |
| `WEB_PORT` | Dev server port | `3000`  |
