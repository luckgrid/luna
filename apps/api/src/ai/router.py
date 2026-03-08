"""AI Agent domain API router."""

from fastapi import APIRouter, status
from fastapi.responses import StreamingResponse

from src.ai import schemas as ai_schemas
from src.ai import service as ai_service

router = APIRouter(prefix="/chat", tags=["AI Agent"])


@router.post(
    "",
    response_class=StreamingResponse,
    status_code=status.HTTP_200_OK,
    description="Chat endpoint with SSE streaming",
    responses={
        status.HTTP_200_OK: {
            "description": "Streaming chat response",
            "content": {"text/event-stream": {}},
        },
    },
)
async def chat(request: ai_schemas.ChatRequest) -> StreamingResponse:
    """Chat endpoint with SSE streaming.

    Provides streaming responses for AI-powered conversations using Pydantic AI.

    Args:
        request: Chat request with message and optional history

    Returns:
        StreamingResponse with SSE format
    """

    async def generate():
        async for chunk in ai_service.chat_stream(
            message=request.message,
            history=[msg.model_dump() for msg in request.history],
        ):
            yield chunk

    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )
