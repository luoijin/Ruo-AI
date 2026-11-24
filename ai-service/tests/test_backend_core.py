import json
import uuid
from datetime import UTC, datetime, timedelta

import pytest
from httpx import ASGITransport, AsyncClient

from app.auth.service import create_access_token, hash_token
from app.core.config import settings
from app.core.database import async_session_maker
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
async def test_concurrent_quota_requests():
    import asyncio

    # Test true concurrent quota increments across multiple concurrent requests (ADR-0007)
    user_id = f"user_concurrent_{uuid.uuid4().hex[:6]}"
    async with async_session_maker() as session:
        user = User(id=user_id, email=f"{user_id}@example.com")
        session.add(user)
        await session.commit()

    async def single_increment():
        async with async_session_maker() as session:
            await check_and_increment_quota(session, user_id=user_id, tokens_in=10, tokens_out=20)

    # Launch 10 concurrent requests simultaneously
    await asyncio.gather(*[single_increment() for _ in range(10)])

    async with async_session_maker() as session:
        today = datetime.now(UTC).date()
        from sqlalchemy import select

        res = await session.execute(select(UsageLog).where(UsageLog.user_id == user_id, UsageLog.usage_date == today))
        log = res.scalars().first()
        assert log is not None
        assert log.request_count == 10
        assert log.tokens_in == 100
        assert log.tokens_out == 200


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
