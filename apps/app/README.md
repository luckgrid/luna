# `app`

Luna interactive/SSR app built with SolidStart, Vite, and Nitro.

## Purpose

The authenticated/interactive app for Luna. Hosts the dashboard shell, feature modules, and client-driven flows. Consumes `@luna/ui` (Solid components) and `@luna/ds` (design system styles).

The static marketing/content site lives at [`apps/web`](../web/README.md).

## Stack

- ⚛️ [SolidStart](https://start.solidjs.com/) — full-stack app framework
- 🧩 [SolidJS](https://www.solidjs.com/) — reactive UI library
- 🔀 [Solid Router](https://docs.solidjs.com/solid-router/) — routing for Solid apps
- ⚡ [Vite](https://vite.dev/) — dev server and build tooling
- 🔥 [Nitro](https://nitro.build/) — server runtime
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) — utility CSS via `@luna/ds`

See root [README Tech Stacks](../../README.md#tech-stacks) for toolchain details.

## Features

- **SSR and streaming** — server-side rendering with fine-grained reactivity
- **Client-side routing** — Solid Router for SPA navigation
- **API integration** — calls to Python API service for AI features
- **Design system** — consumes `@luna/ds` and `@luna/ui`
- **SEO-ready** — meta tags, Open Graph via Nitro config

## Local Development

From the workspace root:

```sh
moon run app:dev
```

From this directory:

```sh
bun run dev
```

Default port: `APP_PORT` (`3000`).

## Build and run

From the workspace root:

```sh
moon run app:build
moon run app:start
```

From this directory:

```sh
bun run build
bun run start
```

Build output: **`dist/`** (Nitro server handler).

## App Configs

- project config: [`moon.yml`](moon.yml)
- app scripts: [`package.json`](package.json)
- Vite config: [`vite.config.ts`](vite.config.ts)
- TypeScript config: [`tsconfig.json`](tsconfig.json)
- environment: root [`.env.local`](../../.env.local)

## Environment Variables

SolidStart is full-stack: **Nitro** serves the SSR app and production Node handler. The **Python API** ([`apps/api`](../api/README.md)) is a separate process on `API_PORT` (default `8000`).

| Variable       | Description                                 | Default                        |
| -------------- | ------------------------------------------- | ------------------------------ |
| `APP_PORT`     | SolidStart dev server port                  | `3000`                         |
| `APP_BASE_URL` | SolidStart app origin (canonical / OG URLs) | `http://localhost:${APP_PORT}` |
| `API_PORT`     | Python FastAPI dev port                     | `8000`                         |
| `API_BASE_URL` | Python FastAPI base URL                     | `http://localhost:${API_PORT}` |

## Project Structure

```text
app/
├── src/
│   ├── components/       # Solid components
│   ├── routes/           # File-based routing
│   ├── utils/            # Utility functions
│   ├── app.config.ts     # SolidStart config
│   ├── app.css           # Global styles
│   └── app.tsx           # App entry
├── public/               # Static assets
└── dist/                 # Build output
```
