"""AI Agent domain service layer."""

from collections.abc import AsyncGenerator
from typing import Any

from pydantic_ai import Agent

from src.ai.config import get_agent_config

config = get_agent_config()

# Configure agent with settings
_agent: Agent | None = None


def get_agent() -> Agent:
    """Get or create the Pydantic AI agent instance."""
    global _agent
    if _agent is None:
        _agent = Agent(config.ai_model)
    return _agent


async def chat_stream(
    message: str,
    history: list[dict[str, Any]],
) -> AsyncGenerator[str, None]:
    """Generate streaming chat response.

    Args:
        message: User message
        history: Conversation history

    Yields:
        Streaming response chunks
    """
    agent = get_agent()

    # Build messages from history
    messages = [{"role": msg["role"], "content": msg["content"]} for msg in history]
    messages.append({"role": "user", "content": message})

    # Run agent with streaming
    async with agent.run_stream(
        message,
        message_history=messages if messages else None,
    ) as result:
        async for chunk in result.stream():
            yield f"data: {chunk}\n\n"

    yield "data: [DONE]\n\n"
