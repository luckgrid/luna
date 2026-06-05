"""AI Agent domain service layer."""

from collections.abc import AsyncGenerator
from typing import Any

from pydantic_ai import Agent
from pydantic_ai.messages import ModelMessage, ModelRequest, ModelResponse, TextPart, UserPromptPart

from ai.config import get_agent_config

config = get_agent_config()

# Configure agent with settings
_agent: Agent | None = None


def get_agent() -> Agent:
    """Get or create the Pydantic AI agent instance."""
    global _agent
    if _agent is None:
        _agent = Agent(config.ai_model)
    return _agent


def _history_to_model_messages(history: list[dict[str, Any]]) -> list[ModelMessage]:
    """Turn OpenAI-style {role, content} turns into Pydantic AI model messages."""
    msgs: list[ModelMessage] = []
    for item in history:
        role = item.get("role")
        content = str(item.get("content", ""))
        if role == "user":
            msgs.append(ModelRequest(parts=[UserPromptPart(content=content)]))
        elif role == "assistant":
            msgs.append(ModelResponse(parts=[TextPart(content=content)]))
    return msgs


async def chat_stream(
    message: str,
    history: list[dict[str, Any]],
) -> AsyncGenerator[str, None]:
    """Generate streaming chat response.

    Args:
        message: Current user message (not repeated in ``history``).
        history: Prior turns only, as ``{"role": "user"|"assistant", "content": str}``.

    Yields:
        ``text/event-stream`` lines (``data: ...``), then ``data: [DONE]``.
    """
    agent = get_agent()
    model_history = _history_to_model_messages(history) or None

    async with agent.run_stream(message, message_history=model_history) as result:
        async for chunk in result.stream():
            text = chunk if isinstance(chunk, str) else str(chunk)
            yield f"data: {text}\n\n"

    yield "data: [DONE]\n\n"
