# `web`

Luna marketing web app built with SolidStart, Vite, and Nitro.

## Purpose

This app is the public-facing marketing site for the Luna monorepo.

## Local Development

From the workspace root:

```sh
# run only this app
moon run web:dev

# or run all application-layer dev tasks
bun run dev
```

From this app directory:

```sh
bun run dev
```

## Build and run

From the workspace root:

```sh
moon run web:build
moon run web:start
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
- environment config: [`.env.local`](.env.local)

## Environment Variables

The web app uses Nitro's public runtime config for cross-service communication:

| Variable       | Description                | Default                 |
| -------------- | -------------------------- | ----------------------- |
| `API_BASE_URL` | API server URL for AI chat | `http://localhost:8080` |

Nitro exposes public config to the client as `import.meta.env.NITRO_PUBLIC_*` without requiring the `VITE_` prefix.

Environment variables are loaded from the root `.env.local` via moon's `envFile` option in `moon.yml`, then passed to Nitro's `runtimeConfig.public` in `vite.config.ts`.
