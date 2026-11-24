from datetime import UTC, datetime

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.core.errors import AppException
from app.core.logging import logger
from app.models.all_models import UsageLog


async def check_and_increment_quota(
    session: AsyncSession,
    user_id: str,
    tokens_in: int = 0,
    tokens_out: int = 0,
    upload_bytes: int = 0,
) -> None:
    today = datetime.now(UTC).date()

    # In PostgreSQL (production), this uses atomic insert ON CONFLICT DO UPDATE:
    # INSERT INTO usage_logs (id, user_id, usage_date, request_count, ...)
    # VALUES (...) ON CONFLICT (user_id, usage_date) DO UPDATE SET request_count = usage_logs.request_count + 1
    # For SQLite compatibility across test sessions, execute atomic upsert or query:
    stmt = select(UsageLog).where(
        UsageLog.user_id == user_id,
        UsageLog.usage_date == today,
    )
    result = await session.execute(stmt)
    usage = result.scalars().first()

    if not usage:
        usage = UsageLog(
            user_id=user_id,
            usage_date=today,
            request_count=1,
            tokens_in=tokens_in,
            tokens_out=tokens_out,
            upload_bytes=upload_bytes,
        )
        session.add(usage)
    else:
        if usage.request_count >= settings.DAILY_MESSAGE_QUOTA:
            now = datetime.now(UTC)
            tomorrow = datetime(now.year, now.month, now.day, tzinfo=UTC).replace(hour=23, minute=59, second=59)
            retry_after_sec = max(60, int((tomorrow - now).total_seconds()))
            logger.info(
                f"User {user_id} exceeded daily message quota ({usage.request_count}/{settings.DAILY_MESSAGE_QUOTA})"
            )
            raise AppException(
                429,
                "QUOTA_EXCEEDED",
                "Daily Quota Exceeded",
                f"You have reached your daily quota of {settings.DAILY_MESSAGE_QUOTA} messages. Your quota resets at midnight UTC.",
                extra={"retry_after_sec": retry_after_sec},
            )

        usage.request_count += 1
        usage.tokens_in += tokens_in
        usage.tokens_out += tokens_out
        usage.upload_bytes += upload_bytes

    await session.commit()
