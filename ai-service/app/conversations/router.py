from datetime import datetime

from fastapi import APIRouter, Depends, Query, Request
from fastapi.responses import StreamingResponse
from sqlalchemy.ext.asyncio import AsyncSession

from app.conversations.schemas import (
    ConversationCreate,
    ConversationResponse,
    ConversationUpdate,
    MessageItem,
    SendMessageRequest,
)
from app.conversations.service import (
    create_conversation,
    delete_conversation,
    get_conversation,
    get_current_user_id,
    list_conversations,
    list_messages,
    stream_chat_messages,
    update_conversation,
)
from app.core.database import get_db_session
from app.core.errors import AppException

router = APIRouter(prefix="/conversations", tags=["conversations"])


@router.get("", response_model=list[ConversationResponse])
async def get_conversations(
    request: Request,
    q: str | None = Query(default=None),
    limit: int = Query(default=50, ge=1, le=100),
    before_updated_at: datetime | None = Query(default=None),
    before_id: str | None = Query(default=None),
    include_deleted: bool = Query(default=False),
    session: AsyncSession = Depends(get_db_session),
):
    if (before_updated_at is None) != (before_id is None):
        raise AppException(422, "INVALID_CURSOR", "Invalid Cursor", "Both conversation cursor fields are required.")
    user_id = await get_current_user_id(request)
    return await list_conversations(
        session,
        user_id,
        query=q,
        limit=limit,
        before_updated_at=before_updated_at,
        before_id=before_id,
        include_deleted=include_deleted,
    )


@router.post("", response_model=ConversationResponse)
async def post_conversation(
    request: Request,
    data: ConversationCreate,
    session: AsyncSession = Depends(get_db_session),
):
    user_id = await get_current_user_id(request)
    return await create_conversation(session, user_id, data)


@router.get("/{conversation_id}", response_model=ConversationResponse)
async def get_conversation_detail(
    request: Request,
    conversation_id: str,
    session: AsyncSession = Depends(get_db_session),
):
    user_id = await get_current_user_id(request)
    return await get_conversation(session, user_id, conversation_id)


@router.get("/{conversation_id}/messages", response_model=list[MessageItem])
async def get_conversation_messages(
    request: Request,
    conversation_id: str,
    limit: int = Query(default=50, ge=1, le=100),
    before_created_at: datetime | None = Query(default=None),
    before_id: str | None = Query(default=None),
    session: AsyncSession = Depends(get_db_session),
):
    if (before_created_at is None) != (before_id is None):
        raise AppException(422, "INVALID_CURSOR", "Invalid Cursor", "Both message cursor fields are required.")
    user_id = await get_current_user_id(request)
    return await list_messages(
        session,
        user_id,
        conversation_id,
        limit,
        before_created_at=before_created_at,
        before_id=before_id,
    )


@router.patch("/{conversation_id}", response_model=ConversationResponse)
async def patch_conversation(
    request: Request,
    conversation_id: str,
    data: ConversationUpdate,
    session: AsyncSession = Depends(get_db_session),
):
    user_id = await get_current_user_id(request)
    return await update_conversation(session, user_id, conversation_id, data)


@router.delete("/{conversation_id}")
async def remove_conversation(
    request: Request,
    conversation_id: str,
    session: AsyncSession = Depends(get_db_session),
):
    user_id = await get_current_user_id(request)
    await delete_conversation(session, user_id, conversation_id)
    return {"status": "ok", "message": "Conversation deleted"}


@router.post("/{conversation_id}/messages")
async def post_message_stream(
    request: Request,
    conversation_id: str,
    data: SendMessageRequest,
    session: AsyncSession = Depends(get_db_session),
):
    user_id = await get_current_user_id(request)
    generator = stream_chat_messages(session, user_id, conversation_id, data)
    return StreamingResponse(
        generator,
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )
