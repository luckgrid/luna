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

Default port: `APP_PORT` (`3001`).

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

The app uses Nitro's runtime config for cross-service communication:

| Variable       | Description     | Default                 |
| -------------- | --------------- | ----------------------- |
| `APP_PORT`     | Dev server port | `3001`                  |
| `API_BASE_URL` | Backend API URL | `http://localhost:8080` |

Environment variables are loaded from the root `.env.local` via moon's
`envFile` option, then passed to Nitro's `runtimeConfig` in `vite.config.ts`.
