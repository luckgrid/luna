# `api`

FastAPI + Pydantic AI backend for Luna.

## Purpose

The Python API service for Luna's AI features and backend logic. Provides REST endpoints, Pydantic validation, and AI agent patterns. Communicates with the SolidStart app (`apps/app`) and static site (`apps/web`) as separate processes.

## Stack

- 🐍 [Python](https://www.python.org/) — runtime
- 🚀 [FastAPI](https://fastapi.tiangolo.com/) — API framework
- 🤖 [Pydantic AI](https://ai.pydantic.dev/) — AI agent patterns
- ✅ [Pydantic](https://docs.pydantic.dev/) — schemas and validation
- ⚙️ [pydantic-settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/) — environment-driven settings
- 🌐 [Uvicorn](https://www.uvicorn.org/) — ASGI server

See root [README Tech Stacks](../../README.md#tech-stacks) for toolchain details.

## Features

- **REST endpoints** — health check, AI chat with SSE streaming
- **Pydantic models** — request/response validation
- **AI integration** — Pydantic AI agent patterns for backend AI features
- **CORS configuration** — cross-service communication with app and web
- **Environment-driven config** — settings via pydantic-settings

## Local Development

From the workspace root:

```sh
moon run api:dev
```

From this directory:

```sh
uv sync
PYTHONPATH=src uv run uvicorn main:app --reload --port 8000
```

Default port: `API_PORT` (`8000`).

## Build and run

From the workspace root:

```sh
moon run api:build
moon run api:start
```

From this directory:

```sh
uv sync
PYTHONPATH=src uv run uvicorn main:app --port 8000
```

## App Configs

- project config: [`moon.yml`](moon.yml)
- Python config: [`pyproject.toml`](pyproject.toml)
- environment: root [`.env.local`](../../.env.local)

## Environment Variables

| Variable         | Description                                  | Default     |
| ---------------- | -------------------------------------------- | ----------- |
| `API_DEBUG`      | Enable debug mode (docs, SQL echo, CORS)     | `false`     |
| `API_HOST`       | Server host                                  | `localhost` |
| `API_PORT`       | Server port                                  | `8000`      |
| `API_BASE_URL`   | API base URL for cross-service communication | -           |
| `APP_BASE_URL`   | SolidStart app origin (CORS)                 | -           |
| `WEB_BASE_URL`   | Static site origin (CORS)                    | -           |
| `DATABASE_URL`   | Database connection string                   | -           |
| `AI_MODEL`       | AI model to use (e.g., `openai:gpt-4o-mini`) | -           |
| `OPENAI_API_KEY` | API key for the AI provider                  | -           |

## Project Structure

```text
api/
├── src/
│   ├── ai/               # AI domain (config, router, schemas, service)
│   ├── config.py         # Application configuration
│   ├── database.py       # Database connection
│   ├── exceptions.py     # Global exceptions
│   ├── main.py           # FastAPI app initialization
│   └── models.py         # Global models
├── tests/                # Test files
└── data.db               # SQLite database (local dev)
```

## API Endpoints

- `GET /health` — Health check
- `GET /docs` — API documentation (Swagger UI)
- `GET /redoc` — API documentation (ReDoc)
- `POST /api/v1/chat` — Chat endpoint with SSE streaming
