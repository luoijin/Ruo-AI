import asyncio
import json
import uuid
from datetime import UTC, datetime, timedelta

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import select

from app.auth.service import create_access_token, hash_token
from app.core.config import (
    DEV_GOOGLE_CLIENT_ID_PLACEHOLDER,
    DEV_JWT_SECRET_PLACEHOLDER,
    Settings,
    settings,
)
from app.core.database import async_session_maker
from app.core.errors import AppException
from app.main import app
from app.models.all_models import EmailOtp, Invite, UsageLog, User
from app.providers.base import ChatRequest
from app.providers.gemini import GeminiProvider
from app.safety.quotas import check_and_increment_quota


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.fixture
async def client():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac


@pytest.mark.asyncio
async def test_health_endpoint(client: AsyncClient):
    resp = await client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "healthy"


@pytest.mark.asyncio
async def test_uninvited_user_rejected_on_google_auth(client: AsyncClient):
    resp = await client.post(
        "/v1/auth/google",
        json={"id_token": f"test:uninvited_{uuid.uuid4().hex[:6]}@example.com:sub123"},
    )
    assert resp.status_code == 403
    assert resp.headers["content-type"] == "application/problem+json"
    data = resp.json()
    assert data["code"] == "INVITE_REQUIRED"


@pytest.mark.asyncio
async def test_invited_user_google_auth_and_token_refresh(client: AsyncClient):
    suffix = uuid.uuid4().hex[:6]
    code = f"INV-{suffix}"
    email = f"student_{suffix}@example.com"
    google_sub = f"sub_{suffix}"

    async with async_session_maker() as session:
        invite = Invite(code=code)
        session.add(invite)
        await session.commit()

    resp = await client.post(
        "/v1/auth/google",
        json={"id_token": f"test:{email}:{google_sub}", "invite_code": code},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert "tokens" in data
    assert data["user"]["email"] == email
    refresh_token = data["tokens"]["refresh_token"]

    ref_resp = await client.post("/v1/auth/refresh", json={"refresh_token": refresh_token})
    assert ref_resp.status_code == 200
    ref_data = ref_resp.json()
    assert "access_token" in ref_data
    assert "refresh_token" in ref_data
    new_refresh = ref_data["refresh_token"]
    assert new_refresh != refresh_token


@pytest.mark.asyncio
async def test_refresh_token_reuse_detection_revokes_all(client: AsyncClient):
    # Test that reusing an already rotated refresh token triggers reuse detection and revokes everything
    suffix = uuid.uuid4().hex[:6]
    code = f"INV-REUSE-{suffix}"
    email = f"reuse_{suffix}@example.com"

    async with async_session_maker() as session:
        invite = Invite(code=code)
        session.add(invite)
        await session.commit()

    # Sign in
    resp = await client.post(
        "/v1/auth/google",
        json={"id_token": f"test:{email}:sub_{suffix}", "invite_code": code},
    )
    assert resp.status_code == 200
    token1 = resp.json()["tokens"]["refresh_token"]

    # First rotation: token1 -> token2
    ref_resp1 = await client.post("/v1/auth/refresh", json={"refresh_token": token1})
    assert ref_resp1.status_code == 200
    token2 = ref_resp1.json()["refresh_token"]

    # Second presentation of token1 (the attacker / reuse):
    ref_resp_attack = await client.post("/v1/auth/refresh", json={"refresh_token": token1})
    assert ref_resp_attack.status_code == 401
    assert ref_resp_attack.json()["code"] == "TOKEN_REVOKED"

    # Now token2 should also be revoked because reuse was detected
    ref_resp2 = await client.post("/v1/auth/refresh", json={"refresh_token": token2})
    assert ref_resp2.status_code == 401
    assert ref_resp2.json()["code"] == "TOKEN_REVOKED"


@pytest.mark.asyncio
async def test_otp_attempt_limits_and_expiry(client: AsyncClient):
    email = f"otp_limits_{uuid.uuid4().hex[:6]}@example.com"
    async with async_session_maker() as session:
        invite = Invite(allowlisted_email=email)
        session.add(invite)
        await session.commit()

    # Request OTP
    req_resp = await client.post("/v1/auth/email/request", json={"email": email})
    assert req_resp.status_code == 200

    # Exhaust attempts
    for _ in range(settings.EMAIL_OTP_MAX_ATTEMPTS):
        v = await client.post("/v1/auth/email/verify", json={"email": email, "code": "000000"})
        assert v.status_code == 400
        assert v.json()["code"] == "INVALID_OTP"

    # Next attempt should be blocked by 429 TOO_MANY_ATTEMPTS
    exhausted_resp = await client.post("/v1/auth/email/verify", json={"email": email, "code": "000000"})
    assert exhausted_resp.status_code == 429
    assert exhausted_resp.json()["code"] == "TOO_MANY_ATTEMPTS"

    # Test expired OTP
    expired_email = f"expired_{uuid.uuid4().hex[:6]}@example.com"
    async with async_session_maker() as session:
        inv = Invite(allowlisted_email=expired_email)
        session.add(inv)
        # Create an already expired OTP
        otp_row = EmailOtp(
            email=expired_email,
            code_hash=hash_token("123456"),
            expires_at=datetime.now(UTC).replace(tzinfo=None) - timedelta(minutes=5),
            attempts=0,
        )
        session.add(otp_row)
        await session.commit()

    exp_resp = await client.post("/v1/auth/email/verify", json={"email": expired_email, "code": "123456"})
    assert exp_resp.status_code == 400
    assert exp_resp.json()["code"] == "EXPIRED_OTP"


@pytest.mark.asyncio
async def test_conversations_and_chat_streaming_with_fake_provider(client: AsyncClient):
    email = f"testchat_{uuid.uuid4().hex[:6]}@example.com"
    async with async_session_maker() as session:
        user = User(email=email)
        session.add(user)
        await session.commit()
        user_id = user.id

    token, _ = create_access_token(user_id)
    headers = {"Authorization": f"Bearer {token}"}

    c_resp = await client.post(
        "/v1/conversations",
        headers=headers,
        json={"title": "Algebra Review", "active_language": "tl-PH"},
    )
    assert c_resp.status_code == 200
    conv_id = c_resp.json()["id"]

    client_msg_id = f"msg-{uuid.uuid4().hex}"
    s_resp = await client.post(
        f"/v1/conversations/{conv_id}/messages",
        headers=headers,
        json={
            "client_msg_id": client_msg_id,
            "content": "Puwede mo ba akong tulungan sa factoring?",
        },
    )
    assert s_resp.status_code == 200
    assert "text/event-stream" in s_resp.headers["content-type"]
    body = s_resp.text
    assert "event: meta" in body
    assert "event: delta" in body
    assert "event: done" in body

    # Retry idempotency: exact same client_msg_id
    retry_resp = await client.post(
        f"/v1/conversations/{conv_id}/messages",
        headers=headers,
        json={
            "client_msg_id": client_msg_id,
            "content": "Puwede mo ba akong tulungan sa factoring?",
        },
    )
    assert retry_resp.status_code == 200
    retry_body = retry_resp.text
    assert "event: meta" in retry_body
    assert '"cached": true' in retry_body


@pytest.mark.asyncio
async def test_self_harm_calm_refusal_moderation(client: AsyncClient):
    email = f"crisis_{uuid.uuid4().hex[:6]}@example.com"
    async with async_session_maker() as session:
        user = User(email=email)
        session.add(user)
        await session.commit()
        user_id = user.id

    token, _ = create_access_token(user_id)
    headers = {"Authorization": f"Bearer {token}"}

    c_resp = await client.post("/v1/conversations", headers=headers, json={"title": "Test Crisis"})
    conv_id = c_resp.json()["id"]

    s_resp = await client.post(
        f"/v1/conversations/{conv_id}/messages",
        headers=headers,
        json={
            "client_msg_id": f"msg-crisis-{uuid.uuid4().hex}",
            "content": "Gusto ko nang magpakamatay dahil sa stress",
        },
    )
    assert s_resp.status_code == 200
    body = s_resp.text
    assert "Hopeline Philippines" in body
    assert "National Center for Mental Health" in body


@pytest.mark.asyncio
async def test_quota_exceeded_and_sse_error(client: AsyncClient):
    email = f"quota_{uuid.uuid4().hex[:6]}@example.com"
    async with async_session_maker() as session:
        user = User(email=email)
        session.add(user)
        await session.commit()
        user_id = user.id

        today = datetime.now(UTC).date()
        log = UsageLog(user_id=user_id, usage_date=today, request_count=settings.DAILY_MESSAGE_QUOTA)
        session.add(log)
        await session.commit()

    token, _ = create_access_token(user_id)
    headers = {"Authorization": f"Bearer {token}"}

    c_resp = await client.post("/v1/conversations", headers=headers, json={"title": "Quota Test"})
    conv_id = c_resp.json()["id"]

    s_resp = await client.post(
        f"/v1/conversations/{conv_id}/messages",
        headers=headers,
        json={
            "client_msg_id": f"msg-quota-{uuid.uuid4().hex}",
            "content": "Hello after quota",
        },
    )
    assert s_resp.status_code == 200
    body = s_resp.text
    assert "event: error" in body
    assert "QUOTA_EXCEEDED" in body
    assert "retry_after_sec" in body


@pytest.mark.asyncio
async def test_concurrent_quota_first_request_of_day():
    # Test concurrent requests on a brand new user with no usage row yet
    user_id = f"user_first_{uuid.uuid4().hex[:6]}"
    async with async_session_maker() as session:
        user = User(id=user_id, email=f"{user_id}@example.com")
        session.add(user)
        await session.commit()

    async def single_increment():
        async with async_session_maker() as session:
            return await check_and_increment_quota(session, user_id=user_id, tokens_in=10, tokens_out=20)

    # 5 concurrent first-time requests
    results = await asyncio.gather(*[single_increment() for _ in range(5)])
    assert len(results) == 5

    async with async_session_maker() as session:
        today = datetime.now(UTC).date()
        res = await session.execute(select(UsageLog).where(UsageLog.user_id == user_id, UsageLog.usage_date == today))
        log = res.scalars().first()
        assert log is not None
        assert log.request_count == 5
        assert log.tokens_in == 50
        assert log.tokens_out == 100


@pytest.mark.asyncio
async def test_concurrent_quota_boundary_race_never_over_allows():
    # Set up user at limit - 1 (e.g. 39 / 40)
    user_id = f"user_boundary_{uuid.uuid4().hex[:6]}"
    async with async_session_maker() as session:
        user = User(id=user_id, email=f"{user_id}@example.com")
        session.add(user)
        today = datetime.now(UTC).date()
        log = UsageLog(
            user_id=user_id,
            usage_date=today,
            request_count=settings.DAILY_MESSAGE_QUOTA - 1,
            tokens_in=100,
            tokens_out=200,
        )
        session.add(log)
        await session.commit()

    async def attempt_increment():
        async with async_session_maker() as session:
            try:
                await check_and_increment_quota(session, user_id=user_id, tokens_in=5, tokens_out=10)
                return "ALLOWED"
            except AppException as e:
                if e.status_code == 429 and e.code == "QUOTA_EXCEEDED":
                    return "REJECTED_429"
                raise

    # 10 concurrent requests racing for the final available slot
    outcomes = await asyncio.gather(*[attempt_increment() for _ in range(10)])

    allowed_count = outcomes.count("ALLOWED")
    rejected_count = outcomes.count("REJECTED_429")

    # Boundary rule: Exactly one request must succeed, all others must receive 429
    assert allowed_count == 1, f"Expected exactly 1 request to be allowed, but got {allowed_count}"
    assert rejected_count == 9, f"Expected 9 requests to be rejected with 429, but got {rejected_count}"

    # Confirm in database that request_count equals exactly the limit and never exceeded it
    async with async_session_maker() as session:
        res = await session.execute(select(UsageLog).where(UsageLog.user_id == user_id, UsageLog.usage_date == today))
        final_log = res.scalars().first()
        assert final_log is not None
        assert final_log.request_count == settings.DAILY_MESSAGE_QUOTA, (
            f"Expected request_count to be {settings.DAILY_MESSAGE_QUOTA}, got {final_log.request_count}"
        )

    # Any subsequent request must immediately fail with 429
    async with async_session_maker() as session:
        with pytest.raises(AppException) as exc_info:
            await check_and_increment_quota(session, user_id=user_id)
        assert exc_info.value.status_code == 429
        assert exc_info.value.code == "QUOTA_EXCEEDED"


def test_production_settings_validation():
    # 1. In production, default dev placeholders must be refused
    with pytest.raises(ValueError, match="JWT_SECRET_KEY must be set to a secure secret"):
        Settings(
            ENVIRONMENT="production",
            JWT_SECRET_KEY=DEV_JWT_SECRET_PLACEHOLDER,
            GOOGLE_CLIENT_ID="valid-google-client-id-123456",
        )

    # 2. In production, empty string JWT_SECRET_KEY must not fall back to dev placeholder; must be refused
    with pytest.raises(ValueError, match="JWT_SECRET_KEY must be set to a secure secret"):
        Settings(
            ENVIRONMENT="production",
            JWT_SECRET_KEY="",
            GOOGLE_CLIENT_ID="valid-google-client-id-123456",
        )

    # 3. In production, short JWT_SECRET_KEY (< 32 chars) must be refused
    with pytest.raises(ValueError, match="at least 32 characters long"):
        Settings(
            ENVIRONMENT="production",
            JWT_SECRET_KEY="short-secret-key-under-32-chars",
            GOOGLE_CLIENT_ID="valid-google-client-id-123456",
        )

    # 4. In production, placeholder GOOGLE_CLIENT_ID must be refused
    with pytest.raises(ValueError, match="GOOGLE_CLIENT_ID must be configured"):
        Settings(
            ENVIRONMENT="production",
            JWT_SECRET_KEY="a-very-secure-secret-key-that-is-at-least-32-characters-long",
            GOOGLE_CLIENT_ID=DEV_GOOGLE_CLIENT_ID_PLACEHOLDER,
        )

    # 5. In production, empty GOOGLE_CLIENT_ID must be refused
    with pytest.raises(ValueError, match="GOOGLE_CLIENT_ID must be configured"):
        Settings(
            ENVIRONMENT="production",
            JWT_SECRET_KEY="a-very-secure-secret-key-that-is-at-least-32-characters-long",
            GOOGLE_CLIENT_ID="",
        )

    # 6. Valid production settings must succeed
    prod_settings = Settings(
        ENVIRONMENT="production",
        JWT_SECRET_KEY="a-very-secure-secret-key-that-is-at-least-32-characters-long",
        GOOGLE_CLIENT_ID="valid-google-client-id-123456",
    )
    assert prod_settings.ENVIRONMENT == "production"


@pytest.mark.asyncio
async def test_gemini_provider_429_retry(monkeypatch):
    # Mock httpx to simulate 429 on first attempt, then success
    provider = GeminiProvider(api_key="test_api_key")
    call_count = 0

    class MockStreamResponse:
        def __init__(self, status_code: int):
            self.status_code = status_code

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass

        async def aiter_lines(self):
            chunk = json.dumps({"candidates": [{"content": {"parts": [{"text": "Hello from Gemini"}]}}]})
            yield f"data: {chunk}"
            done = json.dumps(
                {
                    "candidates": [{"finishReason": "STOP"}],
                    "usageMetadata": {"promptTokenCount": 5, "candidatesTokenCount": 3},
                }
            )
            yield f"data: {done}"

        async def aread(self):
            return b""

    class MockClient:
        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass

        def stream(self, method, url, **kwargs):
            nonlocal call_count
            call_count += 1
            if call_count == 1:
                return MockStreamResponse(429)
            return MockStreamResponse(200)

    import httpx

    monkeypatch.setattr(httpx, "AsyncClient", lambda **kwargs: MockClient())

    chunks = []
    async for chunk in provider.stream_chat(ChatRequest(messages=[{"role": "user", "content": "Hi"}])):
        chunks.append(chunk)

    # First attempt hit 429, retry succeeded
    assert call_count == 2
    assert any("Hello from Gemini" in c.text for c in chunks)
