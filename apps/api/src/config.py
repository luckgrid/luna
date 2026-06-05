"""Global application configuration.

Version is managed in pyproject.toml - update there to bump the version.
Environment variables are sourced from the root .env.local file.
"""

import os
from functools import lru_cache
from importlib.metadata import version as get_pkg_version
from pathlib import Path
from urllib.parse import urlparse, urlunparse

from pydantic import Field, PostgresDsn, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


def get_app_version() -> str:
    """Get version from pyproject.toml."""
    try:
        return get_pkg_version("api")
    except Exception:
        return "0.1.0"


def _loopback_cors_aliases(origin: str) -> list[str]:
    """Return origin plus hostname / IPv4 / IPv6 loopback variants for the same port.

    Browsers treat ``http://localhost:3000``, ``http://127.0.0.1:3000``, and
    ``http://[::1]:3000`` as distinct origins; CORS ``Access-Control-Allow-Origin``
    must match exactly. Dev servers or OS DNS may prefer one form after updates.
    """
    u = origin.strip().rstrip("/")
    if not u:
        return []
    parsed = urlparse(u)
    if parsed.scheme not in ("http", "https"):
        return [u]
    host = (parsed.hostname or "").lower()
    port = parsed.port
    if port is None:
        return [u]
    loopback = {"localhost", "127.0.0.1", "::1", "[::1]"}
    if host not in loopback:
        return [u]
    out: list[str] = []
    for h in ("localhost", "127.0.0.1", "[::1]"):
        netloc = f"{h}:{port}"
        out.append(
            urlunparse(
                (parsed.scheme, netloc, parsed.path or "", "", "", ""),
            ).rstrip("/"),
        )
    return list(dict.fromkeys(out))


def get_env_file_path() -> str:
    """Get path to root .env.local file."""
    # src/config.py — resolve repo root .env.local when run via moon, else app-level
    root_env = Path(__file__).parent.parent.parent.parent / ".env.local"
    if root_env.exists():
        return str(root_env)
    app_env = Path(__file__).parent.parent / ".env.local"
    return str(app_env)


def _anchor_sqlite_url(url: str) -> str:
    """Resolve relative SQLite paths against the api app directory."""
    scheme, sep, path = url.partition(":///")
    if not sep or "sqlite" not in scheme or path.startswith("/"):
        return url
    rel = path.lstrip("./") or "data.db"
    abs_path = (Path(__file__).parent.parent / rel).resolve()
    return f"{scheme}:///{abs_path}"


class Settings(BaseSettings):
    """Application settings loaded from environment variables.

    Environment variables (from root .env.local):
    - API_DEBUG: Enable debug mode (API docs, SQL echo, CORS fallbacks)
    - API_HOST: Server host (default: localhost)
    - API_PORT: Server port (default: 8000; usual Uvicorn convention)
    - API_BASE_URL: API base URL for cross-service communication
    - APP_BASE_URL: SolidStart app URL (CORS)
    - WEB_BASE_URL: Static web site URL (CORS)
    - APP_PORT: SolidStart dev port (CORS debug fallback when base URLs unset)
    - WEB_PORT: Static site dev port (CORS debug fallback when base URLs unset)
    - DATABASE_URL: Database connection string
    """

    model_config = SettingsConfigDict(
        env_file=get_env_file_path(),
        env_file_encoding="utf-8",
        extra="ignore",
    )

    # Server (from API_HOST, API_PORT env vars)
    api_host: str = "localhost"
    api_port: int = 8000
    debug: bool = Field(default=False, validation_alias="API_DEBUG")

    @model_validator(mode="after")
    def sync_debug_env(self) -> "Settings":
        """Expose API debug to libraries that read the generic ``DEBUG`` env var."""
        os.environ["DEBUG"] = "true" if self.debug else "false"
        return self

    @model_validator(mode="after")
    def anchor_database_url(self) -> "Settings":
        """Keep SQLite database files inside the api app regardless of cwd."""
        if isinstance(self.database_url, str) and self.database_url:
            self.database_url = _anchor_sqlite_url(self.database_url)
        return self

    # Dev server ports (APP_PORT, WEB_PORT) — align CORS with vite.config / moon envFile
    app_port: int = 3000
    web_port: int = 3001

    # Base URLs (from API_BASE_URL, WEB_BASE_URL, APP_BASE_URL env vars)
    api_base_url: str = ""
    app_base_url: str = ""
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
        """Browser origins allowed to call this API (SolidStart + static web)."""
        origins: list[str] = []
        for raw in (self.web_base_url, self.app_base_url):
            u = raw.strip()
            if u:
                origins.extend(_loopback_cors_aliases(u))
        if not origins and self.debug:
            for port in (self.app_port, self.web_port):
                origins.extend(_loopback_cors_aliases(f"http://localhost:{port}"))
        return list(dict.fromkeys(origins))

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
