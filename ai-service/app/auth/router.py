from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.schemas import (
    AuthResponse,
    EmailOtpRequest,
    EmailOtpVerifyRequest,
    GoogleAuthRequest,
    RefreshTokenRequest,
    TokenResponse,
)
from app.auth.service import (
    request_email_otp,
    rotate_refresh_token,
    verify_email_otp,
    verify_google_token_and_auth,
)
from app.core.database import get_db_session

router = APIRouter(prefix="/auth", tags=["auth"])


@router.post("/google", response_model=AuthResponse)
async def auth_google(req: GoogleAuthRequest, session: AsyncSession = Depends(get_db_session)):
    return await verify_google_token_and_auth(session, req.id_token, req.invite_code)


@router.post("/email/request")
async def email_request_otp(req: EmailOtpRequest, session: AsyncSession = Depends(get_db_session)):
    await request_email_otp(session, req.email, req.invite_code)
    return {"status": "ok", "message": "Verification code sent"}


@router.post("/email/verify", response_model=AuthResponse)
async def email_verify_otp(req: EmailOtpVerifyRequest, session: AsyncSession = Depends(get_db_session)):
    return await verify_email_otp(session, req.email, req.code)


@router.post("/refresh", response_model=TokenResponse)
async def refresh_tokens(req: RefreshTokenRequest, session: AsyncSession = Depends(get_db_session)):
    return await rotate_refresh_token(session, req.refresh_token)
