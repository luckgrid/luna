"""Global exception handlers."""

from fastapi import HTTPException, Request, status
from fastapi.responses import JSONResponse


class AppException(HTTPException):
    """Base application exception."""

    def __init__(
        self,
        status_code: int,
        detail: str,
        error_code: str | None = None,
    ):
        super().__init__(status_code=status_code, detail=detail)
        self.error_code = error_code


class NotFoundException(AppException):
    """Resource not found exception."""

    def __init__(self, resource: str, identifier: str | int):
        detail = f"{resource} with id '{identifier}' not found"
        super().__init__(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=detail,
            error_code="NOT_FOUND",
        )


class ValidationException(AppException):
    """Validation error exception."""

    def __init__(self, detail: str):
        super().__init__(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=detail,
            error_code="VALIDATION_ERROR",
        )


async def app_exception_handler(request: Request, exc: AppException) -> JSONResponse:
    """Handle application exceptions."""
    content = {
        "error": exc.error_code or "ERROR",
        "detail": exc.detail,
    }
    return JSONResponse(
        status_code=exc.status_code,
        content=content,
    )


async def generic_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """Handle generic exceptions."""
    content = {
        "error": "INTERNAL_ERROR",
        "detail": "An unexpected error occurred",
    }
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content=content,
    )
