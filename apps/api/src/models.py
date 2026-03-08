"""Global Pydantic models."""

from datetime import datetime
from zoneinfo import ZoneInfo

from fastapi.encoders import jsonable_encoder
from pydantic import BaseModel, ConfigDict


def datetime_to_gmt_str(dt: datetime) -> str:
    """Convert datetime to GMT string format."""
    if not dt.tzinfo:
        dt = dt.replace(tzinfo=ZoneInfo("UTC"))
    return dt.strftime("%Y-%m-%dT%H:%M:%S%z")


class CustomModel(BaseModel):
    """Custom base model with common configuration.

    - Serializes all datetime fields to standard format with explicit timezone
    - Provides method to return dict with only serializable fields
    - Allows both alias and field name for population
    """

    model_config = ConfigDict(
        json_encoders={datetime: datetime_to_gmt_str},
        populate_by_name=True,
    )

    def serializable_dict(self, **kwargs) -> dict:
        """Return a dict which contains only serializable fields."""
        default_dict = self.model_dump(**kwargs)
        return jsonable_encoder(default_dict)


class HealthResponse(CustomModel):
    """Health check response."""

    status: str
    version: str


class ErrorResponse(CustomModel):
    """Standard error response."""

    error: str
    detail: str
