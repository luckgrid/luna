"""Tests for the API application."""
import pytest
from httpx import AsyncClient, ASGITransport

from src.main import app


@pytest.fixture
async def client():
    """Create async test client."""
    async with AsyncClient(
        transport=ASGITransport(app=app),
        base_url="http://test",
    ) as client:
        yield client


@pytest.mark.asyncio
async def test_health(client: AsyncClient):
    """Test health check endpoint."""
    response = await client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"
    assert "version" in data


@pytest.mark.asyncio
async def test_docs_available(client: AsyncClient):
    """Test that docs are available in local environment."""
    response = await client.get("/docs")
    assert response.status_code == 200
    assert "text/html" in response.headers.get("content-type", "")


@pytest.mark.asyncio
async def test_openapi_available(client: AsyncClient):
    """Test that OpenAPI spec is available."""
    response = await client.get("/openapi.json")
    assert response.status_code == 200
    data = response.json()
    assert data["info"]["title"] == "API"
    assert "/api/v1/chat" in data["paths"]
