import argparse
import asyncio
import os
import sys
import uuid
from datetime import datetime, timedelta

# Ensure ai-service root is in sys.path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.core.config import settings
from app.models.all_models import Invite

engine = create_async_engine(settings.DATABASE_URL, echo=False)
session_maker = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


async def create_invite(code: str | None = None, email: str | None = None, days: int = 30) -> None:
    async with session_maker() as session:
        if not code and not email:
            code = f"RUO-{uuid.uuid4().hex[:8].upper()}"

        expires_at = datetime.utcnow() + timedelta(days=days) if days > 0 else None

        invite = Invite(
            code=code,
            allowlisted_email=email.lower().strip() if email else None,
            expires_at=expires_at,
        )
        session.add(invite)
        await session.commit()
        print("Invite created successfully:")
        print(f"  ID: {invite.id}")
        if invite.code:
            print(f"  Code: {invite.code}")
        if invite.allowlisted_email:
            print(f"  Allowlisted Email: {invite.allowlisted_email}")
        print(f"  Expires At: {invite.expires_at}")


async def list_invites() -> None:
    async with session_maker() as session:
        result = await session.execute(select(Invite))
        invites = result.scalars().all()
        print(f"Total invites: {len(invites)}")
        for inv in invites:
            status = (
                "USED"
                if inv.used_by
                else ("EXPIRED" if inv.expires_at and inv.expires_at < datetime.utcnow() else "ACTIVE")
            )
            print(f"[{status}] Code: {inv.code} | Email: {inv.allowlisted_email} | UsedBy: {inv.used_by}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Manage Ruo AI beta invites")
    subparsers = parser.add_subparsers(dest="cmd")

    create_parser = subparsers.add_parser("create", help="Create an invite code or allowlisted email")
    create_parser.add_argument("--code", type=str, help="Custom invite code")
    create_parser.add_argument("--email", type=str, help="Allowlisted email")
    create_parser.add_argument("--days", type=int, default=30, help="Valid days (0 for never expires)")

    subparsers.add_parser("list", help="List all invites")

    args = parser.parse_args()
    if args.cmd == "create":
        asyncio.run(create_invite(code=args.code, email=args.email, days=args.days))
    elif args.cmd == "list":
        asyncio.run(list_invites())
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
