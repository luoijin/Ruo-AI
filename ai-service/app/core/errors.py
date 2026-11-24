from typing import Any

from fastapi import Request
from fastapi.responses import JSONResponse


class AppException(Exception):
    def __init__(
        self,
        status_code: int,
        code: str,
        title: str,
        detail: str,
        headers: dict[str, str] | None = None,
        extra: dict[str, Any] | None = None,
    ):
        super().__init__(detail)
        self.status_code = status_code
        self.code = code
        self.title = title
        self.detail = detail
        self.headers = headers or {}
        self.extra = extra or {}


def create_problem_details_response(
    status_code: int,
    code: str,
    title: str,
    detail: str,
    request_id: str = "-",
    headers: dict[str, str] | None = None,
    extra: dict[str, Any] | None = None,
) -> JSONResponse:
    content: dict[str, Any] = {
        "type": f"https://ruo.ai/errors/{code.lower().replace('_', '-')}",
        "title": title,
        "status": status_code,
        "detail": detail,
        "code": code,
        "request_id": request_id,
    }
    if extra:
        content.update(extra)

    resp_headers = {"Content-Type": "application/problem+json"}
    if headers:
        resp_headers.update(headers)
    return JSONResponse(status_code=status_code, content=content, headers=resp_headers)


async def app_exception_handler(request: Request, exc: AppException) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "-")
    return create_problem_details_response(
        status_code=exc.status_code,
        code=exc.code,
        title=exc.title,
        detail=exc.detail,
        request_id=request_id,
        headers=exc.headers,
        extra=exc.extra,
    )
