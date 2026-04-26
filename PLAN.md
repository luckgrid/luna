# Luna Monorepo Template Dev Plan

## Objective

Evolve **Luna** into a clean, open-source monorepo template that supports:

- Multiple app types (static + SSR app)
- Scalable frontend and backend workspaces
- Shared design system and UI primitives
- Clear separation of concerns without over-engineering
- Extensibility for large, modular systems

This is a **foundation template**, not a product.

---

## Apps

### 1. `apps/app` (SolidStart)

Target name for the **interactive/SSR app**.

> Current repo state (per `README.md`): SolidStart currently lives at `apps/web` and will be renamed to `apps/app` as part of the restructure.

#### Responsibilities

- SSR + client interactivity
- Auth boundary (pluggable)
- Dashboard shell layout
- AI terminal placeholder (UI only)
- Feature modules (future expansion)

#### Requirements

- SolidStart latest
- TypeScript strict mode
- Tailwind/CSS via `@luna/ds`
- Shared UI via `@luna/ui`
- Env/config via root `.env.local` (moon `envFile`) + app runtime config

#### Structure

```txt
app/
  src/
    routes/
    components/
    features/        # modular feature domains (future scaling)
    lib/
    styles/
```

#### Notes

- Do not tightly couple to backend beyond a stable HTTP boundary (the repo already includes `apps/api`)
- Keep feature boundaries modular from day one
- Prepare for large-scale route expansion

---

### 2. `apps/web` (Go + `templ` SSG)

New app for **static content + marketing**, generated via Go using [`templ`](https://templ.guide/).

#### Responsibilities

- Static marketing pages
- Markdown-based content (articles, docs, legal) → rendered to static HTML
- SEO + performance optimized output
- Design system styling (Tailwind/CSS output from `@luna/ds`)
- Optional “sprinkles” interactivity where needed (prefer **no JS**; consider Datastar only for specific islands)

#### Requirements

- Go (pinned via repo toolchain)
- `templ` (compile-time HTML components; supports static rendering)
- Markdown rendering (e.g. `goldmark`) and frontmatter parsing
- Static asset pipeline for CSS/images/fonts (copy into output)
- Optional server mode for previews and dynamic pages (same components can be used as handlers)

#### Structure

```txt
web/
  cmd/
    web-ssg/              # generates dist/ from content + templates
      main.go
  content/
    docs/
    articles/
    legal/
  site/
    components/           # templ components (layouts, nav, footer, etc.)
    pages/                # page-level templ components
    render/               # markdown/frontmatter + routing helpers
  public/                 # static assets (favicons, images, etc.)
  dist/                   # generated output (gitignored)
```

#### Notes

- Treat as a **content-first** system; keep templates pure/idempotent (data passed in, no hidden IO).
- Build step: `templ generate` → `go run ./cmd/web-ssg` to write HTML files into `dist/`.
- Markdown content: parse frontmatter into a typed `Post/Page` model, render Markdown to HTML, then embed as a `templ.Component` (unsafe HTML wrapper) during generation.
- Keep client-side JS off by default. If/when interactivity is needed, prefer hypermedia patterns; Datastar is an option for reactive partial updates without building a SPA.
- Hosting: deploy `dist/` anywhere static. If we later add a preview/SSR server, `templ.Handler` can serve pages and a Docker image can bundle `/public` assets.

---

## Workspace Scaling Strategy

### Frontend Scaling

Document common patterns in both apps, extend ds, ui features and create new modules or packages as needed

---

### Backend Scaling

The repo already includes `apps/api` (FastAPI + Pydantic AI) as the backend application-layer project.

If/when backend scope expands beyond a single service, consider introducing a `server/` workspace (or additional `apps/*`) for:

- Additional backend services (workers, cron, realtime, etc.)
- Shared backend libraries (Python packages) with clear boundaries
- Common config patterns (still driven by root `.env.local`)

---

## Tooling

### Package Manager

- **Bun** (workspace-based, Bun-first monorepo)

### Build Orchestration

- `moonrepo`

### Runtime

- `bun` (where applicable)

### Language

- TypeScript (strict)

### Styling

- Tailwind CSS (via design system package)

---

## Naming Conventions

- `app` = authenticated / interactive app
- `web` = static site
- `ds` = design system
- `ui` = UI primitives

Avoid:

- vague names like `core`, `shared`
- deep nesting of packages early

---

## Implementation Steps

### Step 0 — Upgrade Toolchains and Dependencies (Before Feature Work)

Before implementing new features or restructuring apps, upgrade to **latest major versions** and fix any resulting issues so the template stays “green”.

- **Toolchain pins (proto)**:
  - Update pins in `.prototools` (Bun, moon, proto, Python) to latest majors as appropriate.
  - Run `proto install` and ensure tasks still run locally + in CI.
- **JS/TS deps (Bun workspaces)**:
  - Update dependencies across workspaces (prefer updating lockfile once from repo root).
  - Run: lint, format check, and typecheck (`bun run check`) and fix regressions before continuing.
- **Python deps (`apps/api`)**:
  - Sync/upgrade via `uv` and ensure `moon run api:dev` and tests still work.
- **Moon workspace health**:
  - Ensure `moon run :build`, `moon run :dev` (application layer), and `moon run :clean` remain valid.
- **Quality gates**:
  - No new warnings/errors from OXC, TypeScript project references, or Python lint/format tasks.

### Step 1 — Restructure Apps

- Rename current SolidStart marketing app `apps/web` → `apps/app`
- Create new Go + `templ` SSG app → `apps/web`
- Ensure root docs (`README.md`) reflect the transition and new responsibilities:
  - `apps/app`: interactive/SSR app
  - `apps/web`: static marketing/content site (generated)
  - `apps/api`: backend API (already exists)

### Step 2 — Extract Configs

- **Already largely standardized** via root `.env.local` + moon `envFile`.
- If adding stronger validation, choose a per-language approach:
  - TypeScript: zod schema + typed config helpers in the app/packages that need it
  - Python: pydantic-settings (already in the stack) for `apps/api`

### Step 3 — Update Design System

- Update tokens and styles (CSS) where needed (DS already exists as `@luna/ds`)
- Pure and minimal CSS solution
- Integrate into ui and both apps

### Step 4 — Update UI Package

- UI already exists as `@luna/ui` (Solid components, source-first). Extend deliberately.
- Keep DS/UI boundaries clean: `@luna/ds` owns styling primitives; `@luna/ui` owns interactive Solid components.

### Step 5 — Clean Boundaries

- Remove duplicate configs in apps and packages
- Enforce package imports
- Validate build + dev flow

### Step 6 — Prepare for Scale

- Update common data models and collections
- Ensure no tight coupling

---

## Acceptance Criteria

- All application-layer projects build and run independently (`apps/app`, `apps/web`, `apps/api`)
- Shared packages resolve correctly
- No duplicated config across apps
- Design system applied consistently
- Template is minimal but extensible
- New apps/packages can be added without refactor

---

## Next Steps (After This Phase)

- Add auth layer to `app`
- Introduce backend workspace
- Expand UI system
- Add CI/CD + deployment presets

---

## Guiding Principles

- Start minimal, scale intentionally
- Favor composition over abstraction
- Keep boundaries clean
- Avoid premature backend coupling
- Optimize for long-term maintainability
