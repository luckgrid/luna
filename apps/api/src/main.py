"""FastAPI + Pydantic AI backend."""
import os
from typing import Any

from fastapi import FastAPI, SSEEvent
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from pydantic_ai import Agent

app = FastAPI()

# Configure model - uses OpenAI by default, override with env vars
# For local testing, can use Ollama: OllamaModel('llama3.2')
model = os.getenv("AI_MODEL", "openai:gpt-4o-mini")

agent = Agent(model)


class ChatRequest(BaseModel):
    message: str
    history: list[dict[str, Any]] = []


class ChatResponse(BaseModel):
    message: str
    type: str = "response"


@app.get("/health")
async def health():
    return {"status": "ok"}


@app.post("/chat")
async def chat(request: ChatRequest) -> StreamingResponse:
    """Chat endpoint with SSE streaming."""
    
    async def generate():
        # Build messages from history
        messages = []
        for msg in request.history:
            messages.append({"role": msg["role"], "content": msg["content"]})
        messages.append({"role": "user", "content": request.message})
        
        # Run agent with streaming
        async with agent.run_stream(
            request.message,
            message_history=messages if messages else None,
        ) as result:
            async for chunk in result.stream():
                yield f"data: {chunk}\n\n"
        
        yield "data: [DONE]\n\n"
    
    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=3001)
