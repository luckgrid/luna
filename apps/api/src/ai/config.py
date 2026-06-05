"""AI domain configuration.

Environment variables (from root .env.local):
- AI_MODEL: Model to use (e.g., openai:gpt-4o-mini)
- OPENAI_API_KEY: API key for the AI provider
"""

import os
from functools import lru_cache

from config import get_env_file_path
from pydantic_settings import BaseSettings, SettingsConfigDict


class AgentConfig(BaseSettings):
    """Configuration for Pydantic AI agent."""

    model_config = SettingsConfigDict(
        env_file=get_env_file_path(),
        env_file_encoding="utf-8",
        extra="ignore",
    )

    ai_model: str = ""
    openai_api_key: str | None = None

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        if self.openai_api_key:
            os.environ["OPENAI_API_KEY"] = self.openai_api_key


@lru_cache
def get_agent_config() -> AgentConfig:
    """Get cached agent config instance."""
    return AgentConfig()
