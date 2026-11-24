from datetime import UTC, datetime

from sqlalchemy import select
from sqlalchemy.dialects.postgresql import insert as pg_insert
from sqlalchemy.dialects.sqlite import insert as sqlite_insert
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.core.errors import AppException
from app.core.logging import logger
from app.models.all_models import UsageLog
from app.models.common import generate_uuid


async def check_and_increment_quota(
    session: AsyncSession,
    user_id: str,
    tokens_in: int = 0,
    tokens_out: int = 0,
    upload_bytes: int = 0,
) -> None:
    today = datetime.now(UTC).date()

    # Determine database dialect
    bind = session.get_bind()
    dialect_name = bind.dialect.name if bind else "postgresql"

    # Pre-check current usage against daily quota
    check_stmt = select(UsageLog.request_count).where(
        UsageLog.user_id == user_id,
        UsageLog.usage_date == today,
    )
    result = await session.execute(check_stmt)
    current_count = result.scalar() or 0

    if current_count >= settings.DAILY_MESSAGE_QUOTA:
        now = datetime.now(UTC)
        tomorrow = datetime(now.year, now.month, now.day, tzinfo=UTC).replace(hour=23, minute=59, second=59)
        retry_after_sec = max(60, int((tomorrow - now).total_seconds()))
        logger.info(f"User {user_id} exceeded daily message quota ({current_count}/{settings.DAILY_MESSAGE_QUOTA})")
        raise AppException(
            429,
            "QUOTA_EXCEEDED",
            "Daily Quota Exceeded",
            f"You have reached your daily quota of {settings.DAILY_MESSAGE_QUOTA} messages. Your quota resets at midnight UTC.",
            extra={"retry_after_sec": retry_after_sec},
        )

    # Atomic insert-or-update (ADR-0007: Postgres-based quotas via atomic ON CONFLICT DO UPDATE)
    insert_fn = pg_insert if dialect_name == "postgresql" else sqlite_insert
    upsert_stmt = (
        insert_fn(UsageLog)
        .values(
            id=generate_uuid(),
            user_id=user_id,
            usage_date=today,
            request_count=1,
            tokens_in=tokens_in,
            tokens_out=tokens_out,
            upload_bytes=upload_bytes,
        )
        .on_conflict_do_update(
            index_elements=["user_id", "usage_date"],
            set_={
                "request_count": UsageLog.request_count + 1,
                "tokens_in": UsageLog.tokens_in + tokens_in,
                "tokens_out": UsageLog.tokens_out + tokens_out,
                "upload_bytes": UsageLog.upload_bytes + upload_bytes,
            },
        )
    )
    await session.execute(upsert_stmt)
    await session.commit()
