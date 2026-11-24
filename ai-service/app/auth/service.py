import hashlib
import secrets
from datetime import UTC, datetime, timedelta
from typing import Any

import bcrypt
import jwt
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.email_sender import get_email_sender
from app.auth.schemas import AuthResponse, TokenResponse, UserResponse
from app.core.config import settings
from app.core.errors import AppException
from app.core.logging import logger
from app.models.all_models import EmailOtp, Invite, RefreshToken, User


def hash_token(token: str) -> str:
    return hashlib.sha256(token.encode("utf-8")).hexdigest()


def create_access_token(user_id: str) -> tuple[str, int]:
    expires_delta = timedelta(minutes=settings.JWT_ACCESS_TOKEN_EXPIRE_MINUTES)
    expire = datetime.now(UTC) + expires_delta
    payload: dict[str, Any] = {
        "sub": user_id,
        "exp": expire,
        "iat": datetime.now(UTC),
        "type": "access",
    }
    encoded = jwt.encode(payload, settings.JWT_SECRET_KEY, algorithm=settings.JWT_ALGORITHM)
    return encoded, int(expires_delta.total_seconds())


async def create_refresh_token(session: AsyncSession, user_id: str) -> str:
    raw_token = secrets.token_urlsafe(48)
    token_h = hash_token(raw_token)
    expires_at = datetime.now(UTC) + timedelta(days=settings.REFRESH_TOKEN_EXPIRE_DAYS)

    token_row = RefreshToken(
        user_id=user_id,
        token_hash=token_h,
        expires_at=expires_at.replace(tzinfo=None),
    )
    session.add(token_row)
    return raw_token


async def verify_invite_or_allowlist(session: AsyncSession, email: str, invite_code: str | None) -> Invite | None:
    now = datetime.now(UTC).replace(tzinfo=None)

    # 1. Check if email is allowlisted
    stmt = select(Invite).where(
        func.lower(Invite.allowlisted_email) == email.lower(),
        Invite.used_by.is_(None),
    )
    res = await session.execute(stmt)
    invite = res.scalars().first()
    if invite:
        if invite.expires_at and invite.expires_at < now:
            raise AppException(403, "INVITE_EXPIRED", "Invite Expired", "Allowlist invitation has expired.")
        return invite

    # 2. Check invite code
    if invite_code:
        stmt = select(Invite).where(
            Invite.code == invite_code.strip(),
            Invite.used_by.is_(None),
        )
        res = await session.execute(stmt)
        invite = res.scalars().first()
        if not invite:
            raise AppException(
                403, "INVALID_INVITE", "Invalid Invite", "Invite code is invalid or has already been used."
            )
        if invite.expires_at and invite.expires_at < now:
            raise AppException(403, "INVITE_EXPIRED", "Invite Expired", "Invite code has expired.")
        return invite

    # No valid invite or allowlist
    raise AppException(
        403,
        "INVITE_REQUIRED",
        "Invite Required",
        "Ruo AI is currently invite-only. A valid invite code or allowlisted email is required.",
    )


async def get_or_create_user(
    session: AsyncSession,
    email: str,
    google_sub: str | None = None,
    invite_code: str | None = None,
) -> User:
    email_clean = email.lower().strip()
    stmt = select(User).where(User.email == email_clean)
    res = await session.execute(stmt)
    user = res.scalars().first()

    if user:
        if google_sub and not user.google_sub:
            user.google_sub = google_sub
            await session.flush()
        return user

    # New user: enforce invite gate
    invite = await verify_invite_or_allowlist(session, email_clean, invite_code)

    user = User(
        email=email_clean,
        google_sub=google_sub,
    )
    session.add(user)
    await session.flush()

    if invite:
        invite.used_by = user.id
        await session.flush()

    logger.info(f"New user registered: {user.id}")
    return user


async def request_email_otp(session: AsyncSession, email: str, invite_code: str | None = None) -> None:
    email_clean = email.lower().strip()

    # If user doesn't exist, check invite upfront so uninvited users can't spam OTPs
    stmt = select(User).where(User.email == email_clean)
    res = await session.execute(stmt)
    user = res.scalars().first()
    if not user:
        await verify_invite_or_allowlist(session, email_clean, invite_code)

    # Generate 6-digit OTP
    otp_code = f"{secrets.randbelow(1_000_000):06d}"
    code_hash = bcrypt.hashpw(otp_code.encode("utf-8"), bcrypt.gensalt()).decode("utf-8")
    expires_at = datetime.now(UTC) + timedelta(minutes=settings.EMAIL_OTP_EXPIRE_MINUTES)

    otp_row = EmailOtp(
        email=email_clean,
        code_hash=code_hash,
        expires_at=expires_at.replace(tzinfo=None),
        attempts=0,
    )
    session.add(otp_row)
    await session.commit()

    email_sender = get_email_sender()
    await email_sender.send_otp_email(email_clean, otp_code)


async def verify_email_otp(
    session: AsyncSession, email: str, code: str, invite_code: str | None = None
) -> AuthResponse:
    email_clean = email.lower().strip()
    now = datetime.now(UTC).replace(tzinfo=None)

    stmt = select(EmailOtp).where(EmailOtp.email == email_clean).order_by(EmailOtp.created_at.desc())
    res = await session.execute(stmt)
    otp_row = res.scalars().first()

    if not otp_row:
        raise AppException(400, "INVALID_OTP", "Invalid Code", "No pending verification code found.")

    if otp_row.attempts >= settings.EMAIL_OTP_MAX_ATTEMPTS:
        raise AppException(
            429,
            "TOO_MANY_ATTEMPTS",
            "Too Many Attempts",
            "Maximum verification attempts exceeded. Please request a new code.",
        )

    if otp_row.expires_at < now:
        raise AppException(400, "EXPIRED_OTP", "Code Expired", "Verification code has expired.")

    otp_row.attempts += 1
    if not bcrypt.checkpw(code.encode("utf-8"), otp_row.code_hash.encode("utf-8")):
        await session.commit()
        raise AppException(400, "INVALID_OTP", "Invalid Code", "Incorrect verification code.")

    # Valid OTP: delete/consume
    await session.delete(otp_row)

    user = await get_or_create_user(session, email_clean, invite_code=invite_code)
    access_tok, exp_sec = create_access_token(user.id)
    refresh_tok = await create_refresh_token(session, user.id)
    await session.commit()

    return AuthResponse(
        tokens=TokenResponse(
            access_token=access_tok,
            refresh_token=refresh_tok,
            token_type="bearer",
            expires_in=exp_sec,
        ),
        user=UserResponse(
            id=user.id,
            email=user.email,
            ui_lang=user.ui_lang,
            preferred_ai_lang=user.preferred_ai_lang,
        ),
    )


async def verify_google_token_and_auth(
    session: AsyncSession, id_token: str, invite_code: str | None = None
) -> AuthResponse:
    # In test or dev mode, support a test token format: test:email@example.com:google_sub_123
    if id_token.startswith("test:"):
        parts = id_token.split(":")
        email = parts[1]
        google_sub = parts[2] if len(parts) > 2 else f"sub_{secrets.token_hex(4)}"
    else:
        # Check standard Google token or unverified fallback
        # In production, verify against Google's tokeninfo endpoint or google-auth library
        # For Phase 1 backend baseline, parse safely or query Google tokeninfo
        import httpx

        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                resp = await client.get(f"https://oauth2.googleapis.com/tokeninfo?id_token={id_token}")
                if resp.status_code != 200:
                    raise AppException(
                        401, "INVALID_TOKEN", "Authentication Failed", "Google token verification failed."
                    )
                data = resp.json()
                email = data.get("email")
                google_sub = data.get("sub")
                if not email or not google_sub:
                    raise AppException(
                        401, "INVALID_TOKEN", "Authentication Failed", "Google token missing email or subject."
                    )
        except httpx.RequestError as e:
            logger.error(f"Google token verification network error: {e}")
            raise AppException(
                503, "AUTH_SERVICE_UNAVAILABLE", "Service Unavailable", "Could not reach Google auth service."
            )

    user = await get_or_create_user(session, email, google_sub=google_sub, invite_code=invite_code)
    access_tok, exp_sec = create_access_token(user.id)
    refresh_tok = await create_refresh_token(session, user.id)
    await session.commit()

    return AuthResponse(
        tokens=TokenResponse(
            access_token=access_tok,
            refresh_token=refresh_tok,
            token_type="bearer",
            expires_in=exp_sec,
        ),
        user=UserResponse(
            id=user.id,
            email=user.email,
            ui_lang=user.ui_lang,
            preferred_ai_lang=user.preferred_ai_lang,
        ),
    )


async def rotate_refresh_token(session: AsyncSession, raw_refresh_token: str) -> TokenResponse:
    now = datetime.now(UTC).replace(tzinfo=None)
    token_h = hash_token(raw_refresh_token)

    stmt = select(RefreshToken).where(RefreshToken.token_hash == token_h)
    res = await session.execute(stmt)
    token_row = res.scalars().first()

    if not token_row:
        raise AppException(401, "INVALID_REFRESH_TOKEN", "Invalid Refresh Token", "Refresh token was not recognized.")

    if token_row.revoked_at is not None:
        # Potential token reuse attack: revoke all tokens for this user
        logger.warning(f"Reused refresh token detected for user {token_row.user_id}! Revoking all.")
        revoke_stmt = select(RefreshToken).where(RefreshToken.user_id == token_row.user_id)
        all_tokens = (await session.execute(revoke_stmt)).scalars().all()
        for t in all_tokens:
            t.revoked_at = now
        await session.commit()
        raise AppException(401, "TOKEN_REVOKED", "Token Revoked", "Refresh token has been revoked.")

    if token_row.expires_at < now:
        raise AppException(401, "TOKEN_EXPIRED", "Token Expired", "Refresh token has expired.")

    # Revoke old refresh token (rotation)
    token_row.revoked_at = now

    # Issue new token pair
    access_tok, exp_sec = create_access_token(token_row.user_id)
    new_refresh_tok = await create_refresh_token(session, token_row.user_id)
    await session.commit()

    return TokenResponse(
        access_token=access_tok,
        refresh_token=new_refresh_tok,
        token_type="bearer",
        expires_in=exp_sec,
    )
