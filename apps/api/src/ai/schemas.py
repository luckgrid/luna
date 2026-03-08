"""AI Agent domain Pydantic schemas."""

from pydantic import BaseModel, Field


class ChatMessage(BaseModel):
    """Chat message in history."""

    role: str = Field(..., description="Message role: user or assistant")
    content: str = Field(..., description="Message content")


class ChatRequest(BaseModel):
    """Chat request payload."""

    message: str = Field(..., min_length=1, description="User message")
    history: list[ChatMessage] = Field(
        default_factory=list,
        description="Conversation history",
    )


class ChatResponse(BaseModel):
    """Chat response payload."""

    message: str = Field(..., description="Assistant response")
    type: str = Field(default="response", description="Response type")


class StreamingChunk(BaseModel):
    """Streaming chunk for SSE."""

    data: str = Field(..., description="Chunk data")
