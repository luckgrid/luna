"""FastAPI + Pydantic AI backend."""

from contextlib import asynccontextmanager

from fastapi import FastAPI, status
from fastapi.middleware.cors import CORSMiddleware

from src.ai.router import router as ai_router
from src.config import get_settings
from src.database import close_db, init_db
from src.exceptions import (
    ValidationException,
    app_exception_handler,
    generic_exception_handler,
)
from src.models import HealthResponse


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan handler for startup/shutdown events."""
    # Startup
    await init_db()
    yield
    # Shutdown
    await close_db()


settings = get_settings()

app_configs = {
    "title": settings.app_name,
    "version": settings.app_version,
    "description": "Luna AI API - FastAPI + Pydantic AI backend",
    "docs_url": settings.docs_url if settings.show_docs else None,
    "redoc_url": settings.redoc_url if settings.show_docs else None,
    "openapi_url": settings.openapi_url if settings.show_docs else None,
    "lifespan": lifespan,
}

app = FastAPI(**app_configs)

# Exception handlers
app.add_exception_handler(ValidationException, app_exception_handler)
app.add_exception_handler(Exception, generic_exception_handler)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Health check endpoint
@app.get(
    "/health",
    response_model=HealthResponse,
    status_code=status.HTTP_200_OK,
    tags=["Health"],
)
async def health():
    """Health check endpoint."""
    return HealthResponse(status="ok", version=settings.app_version)


# Register routers
app.include_router(ai_router, prefix="/api/v1")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host=settings.api_host, port=settings.api_port)
