# `app`

Luna interactive/SSR app built with SolidStart, Vite, and Nitro.

## Purpose

The authenticated/interactive app for Luna. Hosts the dashboard shell, feature
modules, and any client-driven flows. It is the consumer of `@luna/ui` (Solid
components) and `@luna/ds` (design system styles).

The static marketing/content site lives at [`apps/web`](../web/README.md).

## Local Development

From the workspace root:

```sh
# run only this app
moon run app:dev

# or run all application-layer dev tasks
bun run dev
```

From this app directory:

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

From this app directory:

```sh
bun run build
bun run start
```

## App Configs

- project config: [`moon.yml`](moon.yml)
- app scripts: [`package.json`](package.json)
- Vite config: [`vite.config.ts`](vite.config.ts)
- TypeScript config: [`tsconfig.json`](tsconfig.json)
- environment config: root [`.env.local`](../../.env.local)

## Environment Variables

SolidStart is full-stack: **Nitro** serves the SSR app and production Node
handler. The **Python API** ([`apps/api`](../api/README.md) — FastAPI,
Pydantic AI) is a **separate process** on **`API_PORT` (default 8000)**.

`API_BASE_URL` and `APP_BASE_URL` are wired into Nitro `runtimeConfig` / `public`
and into Vite `define` as `NITRO_PUBLIC_*` values. The API URL is for browser
calls to the Python service (for example from [`src/routes/ai.tsx`](src/routes/ai.tsx));
the app URL drives canonical and Open Graph links via
[`src/app.config.ts`](src/app.config.ts). Neither is the Nitro-internal URL.

When a `*_BASE_URL` is unset, [`vite.config.ts`](vite.config.ts) builds
`http://localhost:<port>` from the matching `*_PORT`.

| Variable       | Description                                   | Default                        |
| -------------- | --------------------------------------------- | ------------------------------ |
| `APP_PORT`     | SolidStart dev server port                    | `3000`                         |
| `APP_BASE_URL` | SolidStart app origin (canonical / OG URLs)   | `http://localhost:${APP_PORT}` |
| `API_PORT`     | Python FastAPI dev port (Uvicorn)             | `8000`                         |
| `API_BASE_URL` | Python FastAPI base URL (separate from Nitro) | `http://localhost:${API_PORT}` |

Environment variables are loaded from the root `.env.local` via moon's
`envFile` option, then passed to Nitro's `runtimeConfig` in
[`vite.config.ts`](vite.config.ts).
