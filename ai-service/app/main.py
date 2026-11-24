import time
import uuid

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware

from app.auth.router import router as auth_router
from app.conversations.router import router as conversations_router
from app.core.config import settings
from app.core.errors import AppException, app_exception_handler
from app.core.logging import logger, setup_logging

setup_logging(settings.LOG_LEVEL)

app = FastAPI(
    title="Ruo AI API",
    version="1.0.0",
    docs_url="/docs" if settings.ENVIRONMENT != "production" else None,
    redoc_url=None,
)

# Exception handlers
app.add_exception_handler(AppException, app_exception_handler)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.middleware("http")
async def add_request_id_and_logging(request: Request, call_next):
    request_id = request.headers.get("X-Request-Id") or str(uuid.uuid4())
    request.state.request_id = request_id

    start_time = time.time()
    response = await call_next(request)
    duration_ms = (time.time() - start_time) * 1000

    # Ensure no message content or auth headers are logged
    logger.info(
        f"{request.method} {request.url.path} -> {response.status_code} ({duration_ms:.1f}ms)",
        extra={"request_id": request_id},
    )

    response.headers["X-Request-Id"] = request_id
    return response


@app.get("/health", tags=["system"])
async def health_check():
    return {
        "status": "healthy",
        "environment": settings.ENVIRONMENT,
        "version": "1.0.0",
    }


# Mount V1 Routers
app.include_router(auth_router, prefix="/v1")
app.include_router(conversations_router, prefix="/v1")
