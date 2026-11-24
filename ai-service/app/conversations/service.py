import json
from collections.abc import AsyncIterator
from datetime import UTC

import jwt
from fastapi import Request
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.conversations.schemas import (
    ConversationCreate,
    ConversationDetailResponse,
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
    session: AsyncSession, user_id: str, query: str | None = None
) -> list[ConversationResponse]:
    stmt = (
        select(Conversation)
        .where(Conversation.user_id == user_id, Conversation.deleted_at.is_(None))
        .order_by(Conversation.updated_at.desc())
    )
    if query:
        stmt = stmt.where(Conversation.title.ilike(f"%{query}%"))

    result = await session.execute(stmt)
    convs = result.scalars().all()
    return [
        ConversationResponse(
            id=c.id,
            title=c.title,
            active_language=c.active_language,
            study_mode_default=c.study_mode_default,
            created_at=c.created_at,
            updated_at=c.updated_at,
        )
        for c in convs
    ]


async def create_conversation(session: AsyncSession, user_id: str, data: ConversationCreate) -> ConversationResponse:
    conv = Conversation(
        user_id=user_id,
        title=data.title,
        active_language=data.active_language,
        study_mode_default=data.study_mode_default,
    )
    session.add(conv)
    await session.commit()
    return ConversationResponse(
        id=conv.id,
        title=conv.title,
        active_language=conv.active_language,
        study_mode_default=conv.study_mode_default,
        created_at=conv.created_at,
        updated_at=conv.updated_at,
    )


async def get_conversation(session: AsyncSession, user_id: str, conversation_id: str) -> ConversationDetailResponse:
    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        raise AppException(404, "CONVERSATION_NOT_FOUND", "Not Found", "Conversation not found.")

    msg_stmt = select(Message).where(Message.conversation_id == conversation_id).order_by(Message.created_at.asc())
    msg_res = await session.execute(msg_stmt)
    messages = msg_res.scalars().all()

    return ConversationDetailResponse(
        id=conv.id,
        title=conv.title,
        active_language=conv.active_language,
        study_mode_default=conv.study_mode_default,
        created_at=conv.created_at,
        updated_at=conv.updated_at,
        messages=[
            MessageItem(
                id=m.id,
                conversation_id=m.conversation_id,
                client_msg_id=m.client_msg_id,
                parent_message_id=m.parent_message_id,
                role=m.role,
                content_md=m.content_md,
                citations_json=m.citations_json,
                study_step=m.study_step,
                language=m.language,
                model_used=m.model_used,
                created_at=m.created_at,
            )
            for m in messages
        ],
    )


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
    return ConversationResponse(
        id=conv.id,
        title=conv.title,
        active_language=conv.active_language,
        study_mode_default=conv.study_mode_default,
        created_at=conv.created_at,
        updated_at=conv.updated_at,
    )


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
    await session.commit()


async def stream_chat_messages(
    session: AsyncSession,
    user_id: str,
    conversation_id: str,
    req: SendMessageRequest,
) -> AsyncIterator[str]:
    # 1. Verify conversation
    stmt = select(Conversation).where(
        Conversation.id == conversation_id,
        Conversation.user_id == user_id,
        Conversation.deleted_at.is_(None),
    )
    result = await session.execute(stmt)
    conv = result.scalars().first()
    if not conv:
        yield f"event: error\ndata: {json.dumps({'code': 'CONVERSATION_NOT_FOUND', 'message': 'Conversation not found'})}\n\n"
        return

    # 2. Check idempotency: does client_msg_id already exist?
    existing_msg_stmt = select(Message).where(Message.client_msg_id == req.client_msg_id)
    res_msg = await session.execute(existing_msg_stmt)
    existing_user_msg = res_msg.scalars().first()

    if existing_user_msg:
        # Client retried the request; replay existing stored response
        assistant_stmt = select(Message).where(
            Message.conversation_id == conversation_id,
            Message.parent_message_id == existing_user_msg.id,
            Message.role == "assistant",
        )
        res_asst = await session.execute(assistant_stmt)
        existing_asst = res_asst.scalars().first()

        meta = {
            "message_id": existing_asst.id if existing_asst else existing_user_msg.id,
            "detected_language": existing_asst.language if existing_asst else conv.active_language,
            "study_step": existing_asst.study_step if existing_asst else None,
            "model": existing_asst.model_used if existing_asst else "cached",
            "fallback": False,
        }
        yield f"event: meta\ndata: {json.dumps(meta)}\n\n"
        if existing_asst:
            yield f"event: delta\ndata: {json.dumps({'text': existing_asst.content_md})}\n\n"
        yield f"event: done\ndata: {json.dumps({'finish_reason': 'stop', 'cached': True})}\n\n"
        return

    # 3. Check and increment daily quota
    try:
        await check_and_increment_quota(session, user_id=user_id)
    except AppException as e:
        yield f"event: error\ndata: {json.dumps({'code': e.code, 'message': e.detail, 'retry_after_sec': e.extra.get('retry_after_sec', 60)})}\n\n"
        return

    # 4. Moderation check
    provider = get_model_provider()
    mod_res = await provider.moderate(req.content)
    if mod_res.get("self_harm"):
        # Calm self-harm refusal path with help info
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
        session.add(user_msg)
        await session.flush()

        asst_msg = Message(
            conversation_id=conversation_id,
            client_msg_id=f"asst_{req.client_msg_id}",
            parent_message_id=user_msg.id,
            role="assistant",
            content_md=self_harm_reply,
            language="tl-PH",
            model_used="safety_moderation",
        )
        session.add(asst_msg)
        await session.commit()

        yield f"event: meta\ndata: {json.dumps({'message_id': asst_msg.id, 'detected_language': 'tl-PH', 'fallback': False})}\n\n"
        yield f"event: delta\ndata: {json.dumps({'text': self_harm_reply})}\n\n"
        yield f"event: done\ndata: {json.dumps({'finish_reason': 'stop'})}\n\n"
        return

    # 5. Save user message to database
    user_msg = Message(
        conversation_id=conversation_id,
        client_msg_id=req.client_msg_id,
        parent_message_id=req.parent_message_id,
        role="user",
        content_md=req.content,
    )
    session.add(user_msg)
    await session.flush()

    # 6. Stream from provider
    history_stmt = select(Message).where(Message.conversation_id == conversation_id).order_by(Message.created_at.asc())
    history_res = await session.execute(history_stmt)
    history = history_res.scalars().all()

    chat_messages = []
    for h in history:
        chat_messages.append({"role": h.role, "content": h.content_md})

    meta = {
        "message_id": user_msg.id,
        "detected_language": conv.active_language,
        "study_step": 1 if req.study_mode else None,
        "model": settings.GEMINI_CHAT_MODEL,
        "fallback": False,
    }
    yield f"event: meta\ndata: {json.dumps(meta)}\n\n"

    collected_content = []

    try:
        async for chunk in provider.stream_chat(
            ChatRequest(
                messages=chat_messages,
                model=settings.GEMINI_CHAT_MODEL,
            )
        ):
            if chunk.text:
                collected_content.append(chunk.text)
                yield f"event: delta\ndata: {json.dumps({'text': chunk.text})}\n\n"

            if chunk.finish_reason:
                yield f"event: done\ndata: {json.dumps({'usage': {'input_tokens': chunk.input_tokens, 'output_tokens': chunk.output_tokens}, 'finish_reason': chunk.finish_reason})}\n\n"

        # Save assistant message to database
        full_asst_text = "".join(collected_content)
        asst_msg = Message(
            conversation_id=conversation_id,
            client_msg_id=f"asst_{req.client_msg_id}",
            parent_message_id=user_msg.id,
            role="assistant",
            content_md=full_asst_text,
            language=conv.active_language,
            model_used=settings.GEMINI_CHAT_MODEL,
        )
        session.add(asst_msg)
        await session.commit()

    except AppException as e:
        yield f"event: error\ndata: {json.dumps({'code': e.code, 'message': e.detail, 'retry_after_sec': e.extra.get('retry_after_sec', 0)})}\n\n"
    except Exception:
        yield f"event: error\ndata: {json.dumps({'code': 'INTERNAL_ERROR', 'message': 'An unexpected error occurred'})}\n\n"
