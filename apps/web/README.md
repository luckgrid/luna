# `web`

Luna static marketing/content site: **Markdown** in `src/content/`, **HTML** in `src/templates/`, filled at build with **[Handlebars](https://handlebarsjs.com/guide/)**. TypeScript under **`src/lib/`** parses content, runs **`Bun.markdown`**, prepares view data, and renders templates — no embedded page markup. **`src/main.ts`** is the CLI entry. Styles: **Vite** + [`@luna/ds`](../../packages/ds/README.md).

**DX:** Edit copy in `src/content/`, structure in `src/templates/`, site logic in `src/lib/` and `src/main.ts`.

Production build is **`bun run build`** (`scripts/build.ts`: static HTML then **Vite** for CSS).

## Content and layouts

| Role           | Markdown file                                                | HTML layout partial               | Output                             |
| -------------- | ------------------------------------------------------------ | --------------------------------- | ---------------------------------- |
| Home / landing | [`src/content/index.md`](src/content/index.md)               | `layouts/default.html` + partials | `/index.html`                      |
| Catalog index  | [`src/content/posts/_index.md`](src/content/posts/_index.md) | `layouts/catalog.html`            | `/posts/index.html`                |
| Article        | `src/content/posts/*.md` or `src/content/<page>.md`          | `layouts/article.html`            | `/posts/<slug>/` or `/<page>.html` |

Optional frontmatter: **`layout`**, **`latest_posts_title`** (home). Reserved: **`index.md`** (home), **`posts/_index.md`** (catalog).

## Layout (`src/` + `src/lib/`)

```text
src/main.ts             CLI entry (import.meta.main), orchestrates build
scripts/build.ts        Runs main.ts then vite build
src/lib/
  types.ts              Post model + Handlebars view models + template handles
  utils.ts              Dates, errno, writePage, copyTree, synthetic pages
  content/
    utils.ts            postURL, outputPath, caption, contentDir
    parse.ts            markdownToHtml, gray-matter, parseDir, taxonomy
  templates/
    render.ts           Load HTML from src/templates, compose pages
    context.ts          View builders from Post + utils dates
```

- **`content/`** — Markdown files → typed posts and HTML bodies.
- **`templates/`** — Handlebars loaders paired with **`src/templates/`** on disk.

## Frontmatter

| Key                  | Purpose                              |
| -------------------- | ------------------------------------ |
| `title`              | Page title                           |
| `description`        | Meta / description                   |
| `date`               | YYYY-MM-DD (post order)              |
| `category`           | Catalog grouping (not in URL)        |
| `tags`               | Article tags                         |
| `slug`               | Optional URL slug                    |
| `layout`             | `default` \| `catalog` \| `article`  |
| `latest_posts_title` | Home: featured posts section heading |

## Raw HTML in Markdown

Options live in [`src/lib/content/parse.ts`](src/lib/content/parse.ts) (`markdownToHtml`, see [Bun markdown docs](https://bun.com/docs/runtime/markdown)).

## Prerequisites

- [`.prototools`](../../.prototools) — `bun run setup` or `proto install`.

## Build and serve

```sh
moon run web:build
moon run web:dev
```

`bun run dev` watches **`src/`** (content, lib, templates, `main.ts`).

## Deployment

`dist/` is static HTML/CSS/assets.

## Environment variables

| Variable   | Description     | Default |
| ---------- | --------------- | ------- |
| `WEB_PORT` | Dev server port | `3001`  |
