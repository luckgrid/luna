"""Global application configuration.

Version is managed in pyproject.toml - update there to bump the version.
Environment variables are sourced from the root .env.local file.
"""

from functools import lru_cache
from importlib.metadata import version as get_pkg_version
from pathlib import Path

from pydantic import PostgresDsn
from pydantic_settings import BaseSettings, SettingsConfigDict


def get_app_version() -> str:
    """Get version from pyproject.toml."""
    try:
        return get_pkg_version("api")
    except Exception:
        return "0.1.0"


def get_env_file_path() -> str:
    """Get path to root .env.local file."""
    # Try root .env.local first (when run via moon), fallback to app-level
    root_env = Path(__file__).parent.parent.parent.parent / ".env.local"
    if root_env.exists():
        return str(root_env)
    # Fallback to app-level .env.local
    app_env = Path(__file__).parent.parent / ".env.local"
    return str(app_env)


class Settings(BaseSettings):
    """Application settings loaded from environment variables.

    Environment variables (from root .env.local):
    - API_HOST: Server host (default: localhost)
    - API_PORT: Server port (default: 8080)
    - API_BASE_URL: API base URL for cross-service communication
    - WEB_BASE_URL: Web app URL for CORS
    - DATABASE_URL: Database connection string
    - DEBUG: Enable debug mode
    """

    model_config = SettingsConfigDict(
        env_file=get_env_file_path(),
        env_file_encoding="utf-8",
        extra="ignore",
    )

    # Server (from API_HOST, API_PORT env vars)
    api_host: str = "localhost"
    api_port: int = 8080
    debug: bool = False

    # Base URLs (from API_BASE_URL, WEB_BASE_URL env vars)
    api_base_url: str = ""
    web_base_url: str = ""

    # Database (from DATABASE_URL env var)
    database_url: PostgresDsn | str = ""

    # Docs
    docs_url: str = "/docs"
    redoc_url: str = "/redoc"
    openapi_url: str = "/openapi.json"

    @property
    def show_docs(self) -> bool:
        """Show API documentation only in debug mode."""
        return self.debug

    @property
    def cors_origins(self) -> list[str]:
        """CORS origins derived from web_base_url."""
        if self.web_base_url:
            return [self.web_base_url]
        return []

    @property
    def app_name(self) -> str:
        return "API"

    @property
    def app_version(self) -> str:
        return get_app_version()


@lru_cache
def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()
