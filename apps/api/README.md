# API

FastAPI + Pydantic AI backend for AI features.

## Setup

```sh
# Install dependencies
cd apps/api
uv sync
```

## Run

```sh
# Via moon (recommended - uses root .env.local)
moon run api:dev

# Or directly with uvicorn
uv run uvicorn src.main:app --reload --port 8000
```

## Configuration

Configuration is managed via environment variables from the root `.env.local` file. This ensures consistent config across all apps in the monorepo.

### Environment Variables

| Variable         | Description                                  | Default     |
| ---------------- | -------------------------------------------- | ----------- |
| `API_HOST`       | Server host                                  | `localhost` |
| `API_PORT`       | Server port                                  | `8000`      |
| `DEBUG`          | Enable debug mode                            | `false`     |
| `API_BASE_URL`   | API base URL for cross-service communication | -           |
| `APP_BASE_URL`   | SolidStart app origin (CORS)                 | -           |
| `WEB_BASE_URL`   | Static site origin (CORS)                    | -           |
| `DATABASE_URL`   | Database connection string                   | -           |
| `AI_MODEL`       | AI model to use (e.g., `openai:gpt-4o-mini`) | -           |
| `OPENAI_API_KEY` | API key for the AI provider                  | -           |

### Root .env.local

All apps share a single `.env.local` in the repository root. This file contains environment-specific values for all services:

```sh
# Ports
API_PORT=8000
APP_PORT=3000
WEB_PORT=3001

# Hosts
API_HOST=localhost
APP_HOST=localhost
WEB_HOST=localhost

# URLs
API_BASE_URL=http://localhost:8000
APP_BASE_URL=http://localhost:3000
WEB_BASE_URL=http://localhost:3001

# Database
DATABASE_URL=sqlite+aiosqlite:///./data.db

# AI
AI_MODEL=openai:gpt-4o-mini
OPENAI_API_KEY=your-api-key-here

# Dev
DEBUG=true
```

Moon automatically loads the root `.env.local` into tasks (`dev`, `start`, `test`) using the `envFile` option in `moon.yml`. This passes all environment variables to the running application.

### App-level .env (Fallback)

The API app also has its own `.env.local` for local development fallback. Variable names should match the root for consistency.

## Project Structure

```text
src/
├── ai/                    # AI domain
│   ├── config.py          # AI-specific configuration
│   ├── router.py          # API endpoints
│   ├── schemas.py         # Pydantic models
│   └── service.py         # Business logic
├── config.py              # Global application configuration
├── database.py            # Database connection
├── exceptions.py          # Global exceptions
├── main.py                # FastAPI app initialization
└── models.py              # Global models
```

## API Endpoints

- `GET /health` - Health check
- `GET /docs` - API documentation (Swagger UI)
- `GET /redoc` - API documentation (ReDoc)
- `POST /api/v1/chat` - Chat endpoint with SSE streaming

## Resources

- [FastAPI Best Practices](https://github.com/zhanymkanov/fastapi-best-practices)
- [Pydantic AI Documentation](https://ai.pydantic.dev/)
- [Pydantic Settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/)
- [Moon Repository Manager](https://moonrepo.dev/docs)
