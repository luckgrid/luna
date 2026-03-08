"""AI Agent domain exceptions."""


class AgentError(Exception):
    """Base exception for agent errors."""

    pass


class AgentNotAvailableError(AgentError):
    """Raised when the AI agent is not available."""

    pass


class InvalidMessageError(AgentError):
    """Raised when the message is invalid."""

    pass
