import asyncio
import json
from collections.abc import AsyncIterator
from datetime import UTC, datetime

import jwt
from fastapi import Request
from sqlalchemy import and_, or_, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession

from app.conversations.schemas import (
    ConversationCreate,
    ConversationResponse,
    ConversationUpdate,
    MessageItem,
    SendMessageRequest,
)
from app.core.config import settings
from app.core.errors import AppException
from app.models.all_models import Conversation, Message
from app.providers.base import ChatRequest
from app.providers.registry import get_model_provider
from app.safety.quotas import check_and_increment_quota


async def get_current_user_id(request: Request) -> str:
    auth_header = request.headers.get("Authorization")
    if not auth_header or not auth_header.startswith("Bearer "):
        raise AppException(401, "UNAUTHORIZED", "Unauthorized", "Missing or invalid bearer token.")

    token = auth_header[7:].strip()
    try:
        payload = jwt.decode(token, settings.JWT_SECRET_KEY, algorithms=[settings.JWT_ALGORITHM])
        user_id = payload.get("sub")
        if not user_id:
            raise AppException(401, "INVALID_TOKEN", "Invalid Token", "Token subject missing.")
        return user_id
    except jwt.PyJWTError:
        raise AppException(401, "INVALID_TOKEN", "Invalid Token", "Token signature or expiration invalid.")


async def list_conversations(
    session: AsyncSession,
    user_id: str,
    query: str | None = None,
    limit: int = 50,
    before_updated_at: datetime | None = None,
    before_id: str | None = None,
    include_deleted: bool = False,
) -> list[ConversationResponse]:
    stmt = select(Conversation).where(Conversation.user_id == user_id)
    if not include_deleted:
        stmt = stmt.where(Conversation.deleted_at.is_(None))
    if query:
        stmt = stmt.where(Conversation.title.ilike(f"%{query}%"))
    if before_updated_at is not None and before_id is not None:
        stmt = stmt.where(
            or_(
                Conversation.updated_at < before_updated_at,
                and_(Conversation.updated_at == before_updated_at, Conversation.id < before_id),
            )
        )

    result = await session.execute(stmt.order_by(Conversation.updated_at.desc(), Conversation.id.desc()).limit(limit))
    convs = result.scalars().all()
    return [_conversation_response(conv) for conv in convs]


def _conversation_response(conv: Conversation) -> ConversationResponse:
    return ConversationResponse(
        id=conv.id,
        title=conv.title,
        active_language=conv.active_language,
        study_mode_default=conv.study_mode_default,
        created_at=conv.created_at,
        updated_at=conv.updated_at,
        deleted_at=conv.deleted_at,
    )


async def create_conversation(session: AsyncSession, user_id: str, data: ConversationCreate) -> ConversationResponse:
    conv = Conversation(
        id=data.conversation_id,
        user_id=user_id,
        title=data.title,
        active_language=data.active_language,
        study_mode_default=data.study_mode_default,
    )
    session.add(conv)
    try:
        await session.commit()
        return _conversation_response(conv)
    except IntegrityError:
        await session.rollback()
        existing = await session.get(Conversation, data.conversation_id)
        if existing is None or existing.user_id != user_id:
            raise AppException(
                409,
                "CONVERSATION_ID_CONFLICT",
                "Conversation ID Conflict",
                "The conversation ID is already in use.",
            ) from None
        return _conversation_response(existing)


async def get_conversation(session: AsyncSession, user_id: str, conversation_id: str) -> ConversationResponse:
    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        raise AppException(404, "CONVERSATION_NOT_FOUND", "Not Found", "Conversation not found.")

    return _conversation_response(conv)


async def list_messages(
    session: AsyncSession,
    user_id: str,
    conversation_id: str,
    limit: int,
    before_created_at: datetime | None = None,
    before_id: str | None = None,
) -> list[MessageItem]:
    conversation = await session.scalar(
        select(Conversation).where(
            Conversation.id == conversation_id,
            Conversation.user_id == user_id,
            Conversation.deleted_at.is_(None),
        )
    )
    if conversation is None:
        raise AppException(404, "CONVERSATION_NOT_FOUND", "Not Found", "Conversation not found.")

    stmt = select(Message).where(Message.conversation_id == conversation_id)
    if before_created_at is not None and before_id is not None:
        stmt = stmt.where(
            or_(
                Message.created_at < before_created_at,
                and_(Message.created_at == before_created_at, Message.id < before_id),
            )
        )
    result = await session.execute(stmt.order_by(Message.created_at.desc(), Message.id.desc()).limit(limit))
    return [
        MessageItem(
            id=message.id,
            conversation_id=message.conversation_id,
            client_msg_id=message.client_msg_id,
            parent_message_id=message.parent_message_id,
            role=message.role,
            content_md=message.content_md,
            citations_json=message.citations_json,
            study_step=message.study_step,
            language=message.language,
            model_used=message.model_used,
            status=message.status,
            created_at=message.created_at,
        )
        for message in result.scalars().all()
    ]


async def update_conversation(
    session: AsyncSession, user_id: str, conversation_id: str, data: ConversationUpdate
) -> ConversationResponse:
    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        raise AppException(404, "CONVERSATION_NOT_FOUND", "Not Found", "Conversation not found.")

    if data.title is not None:
        conv.title = data.title
    if data.active_language is not None:
        conv.active_language = data.active_language
    if data.study_mode_default is not None:
        conv.study_mode_default = data.study_mode_default

    await session.commit()
    return _conversation_response(conv)


async def delete_conversation(session: AsyncSession, user_id: str, conversation_id: str) -> None:
    from datetime import datetime

    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        raise AppException(404, "CONVERSATION_NOT_FOUND", "Not Found", "Conversation not found.")

    conv.deleted_at = datetime.now(UTC).replace(tzinfo=None)
    conv.updated_at = conv.deleted_at
    await session.commit()


async def stream_chat_messages(
    session: AsyncSession,
    user_id: str,
    conversation_id: str,
    req: SendMessageRequest,
) -> AsyncIterator[str]:
    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        yield _sse("error", {"code": "CONVERSATION_NOT_FOUND", "message": "Conversation not found", "retryable": False})
        return

    provider = get_model_provider()
    user_msg = await session.scalar(select(Message).where(Message.client_msg_id == req.client_msg_id))
    assistant_msg: Message | None = None
    if user_msg is not None:
        if user_msg.conversation_id != conversation_id or user_msg.content_md != req.content:
            yield _sse("error", {"code": "IDEMPOTENCY_CONFLICT", "message": "Message ID conflict", "retryable": False})
            return
        assistant_msg = await session.scalar(
            select(Message).where(Message.client_msg_id == f"asst_{req.client_msg_id}")
        )
        if assistant_msg is None:
            yield _sse(
                "error",
                {
                    "code": "STREAM_IN_PROGRESS",
                    "message": "The original request is still in progress.",
                    "retryable": True,
                    "retry_after_sec": 1,
                },
            )
            return
        if assistant_msg.status == "complete":
            yield _sse(
                "meta",
                {
                    "client_msg_id": req.client_msg_id,
                    "user_message_id": user_msg.id,
                    "assistant_message_id": assistant_msg.id,
                    "detected_language": assistant_msg.language or conv.active_language,
                    "study_step": assistant_msg.study_step,
                    "model": assistant_msg.model_used or "cached",
                    "fallback": False,
                    "replayed": True,
                },
            )
            if assistant_msg.content_md:
                yield _sse("delta", {"text": assistant_msg.content_md})
            yield _sse(
                "done",
                {
                    "client_msg_id": req.client_msg_id,
                    "assistant_message_id": assistant_msg.id,
                    "replayed": True,
                    "finish_reason": "stop",
                },
            )
            return
        if assistant_msg.status == "streaming":
            yield _sse(
                "error",
                {
                    "code": "STREAM_IN_PROGRESS",
                    "message": "The original request is still in progress.",
                    "retryable": True,
                    "retry_after_sec": 1,
                },
            )
            return
        assistant_msg.status = "streaming"
        assistant_msg.content_md = ""
        assistant_msg.updated_at = datetime.now(UTC).replace(tzinfo=None)
        await session.commit()
    else:
        try:
            await check_and_increment_quota(session, user_id=user_id, commit=False)
        except AppException as exc:
            yield _sse(
                "error",
                {
                    "code": exc.code,
                    "message": "The request could not be completed.",
                    "retryable": exc.code != "QUOTA_EXCEEDED",
                    "retry_after_sec": exc.extra.get("retry_after_sec"),
                },
            )
            return

        mod_res = await provider.moderate(req.content)
        if mod_res.get("self_harm"):
            self_harm_reply = (
                "Naririnig kita at mahalaga ang buhay mo. Kung ikaw o ang kakilala mo ay dumaranas ng matinding kalungkutan "
                "o pagsubok, mangyaring humingi ng tulong:\n\n"
                "📞 **Hopeline Philippines:** 177 o (02) 8804-4673 / 0917-558-4673\n"
                "📞 **National Center for Mental Health Crisis Hotline:** 1553 (Luzon-wide toll-free) o 0917-899-8727\n\n"
                "Hindi ka nag-iisa. Nandiyan ang mga taong handang makinig at tumulong sa iyo."
            )
            user_msg = Message(
                conversation_id=conversation_id,
                client_msg_id=req.client_msg_id,
                parent_message_id=req.parent_message_id,
                role="user",
                content_md=req.content,
            )
            assistant_msg = Message(
                conversation_id=conversation_id,
                client_msg_id=f"asst_{req.client_msg_id}",
                parent_message_id=None,
                role="assistant",
                content_md=self_harm_reply,
                language="tl-PH",
                model_used="safety_moderation",
                status="complete",
            )
            session.add_all([user_msg, assistant_msg])
            try:
                await session.flush()
                assistant_msg.parent_message_id = user_msg.id
                await session.commit()
            except IntegrityError:
                await session.rollback()
                yield _sse(
                    "error",
                    {
                        "code": "STREAM_IN_PROGRESS",
                        "message": "The original request is still in progress.",
                        "retryable": True,
                        "retry_after_sec": 1,
                    },
                )
                return

            yield _sse(
                "meta",
                {
                    "client_msg_id": req.client_msg_id,
                    "user_message_id": user_msg.id,
                    "assistant_message_id": assistant_msg.id,
                    "detected_language": "tl-PH",
                    "study_step": None,
                    "model": "safety_moderation",
                    "fallback": False,
                    "replayed": False,
                },
            )
            yield _sse("delta", {"text": self_harm_reply})
            yield _sse(
                "done",
                {
                    "client_msg_id": req.client_msg_id,
                    "assistant_message_id": assistant_msg.id,
                    "replayed": False,
                    "finish_reason": "stop",
                },
            )
            return

        user_msg = Message(
            conversation_id=conversation_id,
            client_msg_id=req.client_msg_id,
            parent_message_id=req.parent_message_id,
            role="user",
            content_md=req.content,
            status="complete",
        )
        assistant_msg = Message(
            conversation_id=conversation_id,
            client_msg_id=f"asst_{req.client_msg_id}",
            parent_message_id=None,
            role="assistant",
            content_md="",
            language=conv.active_language,
            model_used=settings.GEMINI_CHAT_MODEL,
            status="streaming",
        )
        session.add_all([user_msg, assistant_msg])
        try:
            await session.flush()
            assistant_msg.parent_message_id = user_msg.id
            await session.commit()
        except IntegrityError:
            await session.rollback()
            existing_user = await session.scalar(select(Message).where(Message.client_msg_id == req.client_msg_id))
            existing_assistant = await session.scalar(
                select(Message).where(Message.client_msg_id == f"asst_{req.client_msg_id}")
            )
            if existing_user and existing_assistant and existing_assistant.status == "complete":
                yield _sse(
                    "meta",
                    {
                        "client_msg_id": req.client_msg_id,
                        "user_message_id": existing_user.id,
                        "assistant_message_id": existing_assistant.id,
                        "detected_language": existing_assistant.language or conv.active_language,
                        "study_step": existing_assistant.study_step,
                        "model": existing_assistant.model_used or "cached",
                        "fallback": False,
                        "replayed": True,
                    },
                )
                if existing_assistant.content_md:
                    yield _sse("delta", {"text": existing_assistant.content_md})
                yield _sse(
                    "done",
                    {
                        "client_msg_id": req.client_msg_id,
                        "assistant_message_id": existing_assistant.id,
                        "replayed": True,
                        "finish_reason": "stop",
                    },
                )
            else:
                yield _sse(
                    "error",
                    {
                        "code": "STREAM_IN_PROGRESS",
                        "message": "The original request is still in progress.",
                        "retryable": True,
                        "retry_after_sec": 1,
                    },
                )
            return

    history_stmt = (
        select(Message)
        .where(
            Message.conversation_id == conversation_id,
            Message.id != assistant_msg.id,
            Message.status == "complete",
        )
        .order_by(Message.created_at.asc(), Message.id.asc())
    )
    history_res = await session.execute(history_stmt)
    chat_messages = [{"role": message.role, "content": message.content_md} for message in history_res.scalars().all()]
    meta = {
        "client_msg_id": req.client_msg_id,
        "user_message_id": user_msg.id,
        "assistant_message_id": assistant_msg.id,
        "detected_language": conv.active_language,
        "study_step": 1 if req.study_mode else None,
        "model": settings.GEMINI_CHAT_MODEL,
        "fallback": False,
        "replayed": False,
    }
    yield _sse("meta", meta)

    collected_content: list[str] = []
    usage = {"input_tokens": 0, "output_tokens": 0}
    finish_reason = "stop"
    try:
        async for chunk in provider.stream_chat(ChatRequest(messages=chat_messages, model=settings.GEMINI_CHAT_MODEL)):
            if chunk.text:
                collected_content.append(chunk.text)
                yield _sse("delta", {"text": chunk.text})
            if chunk.finish_reason:
                finish_reason = chunk.finish_reason
                usage = {"input_tokens": chunk.input_tokens, "output_tokens": chunk.output_tokens}

        assistant_msg.content_md = "".join(collected_content)
        assistant_msg.status = "complete"
        assistant_msg.updated_at = datetime.now(UTC).replace(tzinfo=None)
        await session.commit()
        yield _sse(
            "done",
            {
                "client_msg_id": req.client_msg_id,
                "assistant_message_id": assistant_msg.id,
                "replayed": False,
                "usage": usage,
                "finish_reason": finish_reason,
            },
        )
    except asyncio.CancelledError:
        assistant_msg.status = "failed"
        assistant_msg.updated_at = datetime.now(UTC).replace(tzinfo=None)
        await asyncio.shield(session.commit())
        raise
    except AppException as exc:
        assistant_msg.status = "failed"
        assistant_msg.updated_at = datetime.now(UTC).replace(tzinfo=None)
        await session.commit()
        yield _sse(
            "error",
            {
                "code": exc.code,
                "message": "The request could not be completed.",
                "retryable": exc.code != "QUOTA_EXCEEDED",
                "retry_after_sec": exc.extra.get("retry_after_sec"),
            },
        )
    except Exception:
        assistant_msg.status = "failed"
        assistant_msg.updated_at = datetime.now(UTC).replace(tzinfo=None)
        await session.commit()
        yield _sse(
            "error",
            {"code": "INTERNAL_ERROR", "message": "An unexpected error occurred.", "retryable": True},
        )


def _sse(event: str, data: dict[str, object]) -> str:
    return f"event: {event}\ndata: {json.dumps(data)}\n\n"
