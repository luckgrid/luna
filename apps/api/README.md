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
# Run the API server
uv run python src/main.py

# Or with uvicorn directly
uv run uvicorn src.main:app --reload --port 3001
```

## Environment

- `AI_MODEL` - Model to use (default: `openai:gpt-4o-mini`)
- `OPENAI_API_KEY` - OpenAI API key (required for OpenAI models)
