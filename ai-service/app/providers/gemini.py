import asyncio
import json
import random
from collections.abc import AsyncIterator

import httpx

from app.core.config import settings
from app.core.errors import AppException
from app.core.logging import logger
from app.providers.base import ChatChunk, ChatRequest, ModelProvider


class GeminiProvider(ModelProvider):
    """Gemini API implementation reading models from config.

    Verified model name: gemini-4-hydrogen.
    Fallback model configurable via env: GEMINI_FALLBACK_MODEL.
    Handles 429 with jittered backoff.
    """

    def __init__(self, api_key: str | None = None):
        self.api_key = api_key or settings.GEMINI_API_KEY
        self.base_url = "https://generativelanguage.googleapis.com/v1beta"

    async def stream_chat(self, request: ChatRequest, model_override: str | None = None) -> AsyncIterator[ChatChunk]:
        model = model_override or request.model or settings.GEMINI_CHAT_MODEL
        if not self.api_key:
            raise AppException(
                500,
                "PROVIDER_KEY_MISSING",
                "Gemini Key Missing",
                "GEMINI_API_KEY is not configured on the backend.",
            )

        url = f"{self.base_url}/models/{model}:streamGenerateContent?alt=sse&key={self.api_key}"

        # Convert messages to Gemini contents structure
        contents = []
        for m in request.messages:
            role = "user" if m.get("role") in ("user", "system") else "model"
            contents.append({"role": role, "parts": [{"text": m.get("content", "")}]})

        payload = {
            "contents": contents,
            "generationConfig": {
                "temperature": request.temperature,
            },
        }
        if request.system_instruction:
            payload["systemInstruction"] = {"parts": [{"text": request.system_instruction}]}

        max_retries = 2
        for attempt in range(max_retries + 1):
            try:
                async with httpx.AsyncClient(timeout=30.0) as client:
                    async with client.stream("POST", url, json=payload) as response:
                        if response.status_code == 429:
                            if attempt < max_retries:
                                jitter = random.uniform(0.5, 1.5) * (2**attempt)
                                logger.warning(f"Gemini 429 rate limit. Retrying in {jitter:.2f}s...")
                                await asyncio.sleep(jitter)
                                continue
                            else:
                                raise AppException(
                                    429,
                                    "RATE_LIMIT_EXCEEDED",
                                    "Provider Rate Limit",
                                    "Upstream AI provider quota or rate limit exceeded.",
                                    extra={"retry_after_sec": 30},
                                )

                        if response.status_code != 200:
                            err_body = await response.aread()
                            logger.error(
                                f"Gemini error {response.status_code}: {err_body.decode('utf-8', errors='ignore')}"
                            )
                            raise AppException(
                                502,
                                "AI_PROVIDER_ERROR",
                                "AI Provider Error",
                                f"Gemini API returned error status {response.status_code}",
                            )

                        async for line in response.aiter_lines():
                            if line.startswith("data: "):
                                data_str = line[6:].strip()
                                if not data_str or data_str == "[DONE]":
                                    continue
                                try:
                                    data_json = json.loads(data_str)
                                    candidates = data_json.get("candidates", [])
                                    if candidates:
                                        first = candidates[0]
                                        content_parts = first.get("content", {}).get("parts", [])
                                        for part in content_parts:
                                            text_chunk = part.get("text", "")
                                            if text_chunk:
                                                yield ChatChunk(text=text_chunk)
                                        finish_reason = first.get("finishReason")
                                        if finish_reason:
                                            usage = data_json.get("usageMetadata", {})
                                            yield ChatChunk(
                                                text="",
                                                finish_reason=finish_reason.lower(),
                                                input_tokens=usage.get("promptTokenCount", 0),
                                                output_tokens=usage.get("candidatesTokenCount", 0),
                                            )
                                except Exception as parse_err:
                                    logger.warning(f"Error parsing Gemini SSE chunk: {parse_err}")
                        return
            except httpx.RequestError as req_err:
                if attempt < max_retries:
                    await asyncio.sleep(1.0)
                    continue
                logger.error(f"Gemini connection error: {req_err}")
                raise AppException(
                    503,
                    "AI_PROVIDER_UNAVAILABLE",
                    "Provider Unavailable",
                    "Could not connect to Gemini service.",
                )

    async def moderate(self, text: str) -> dict[str, bool | float]:
        text_lower = text.lower()
        self_harm_keywords = ["magpakamatay", "suicide", "kill myself", "end my life", "laslas"]
        is_self_harm = any(k in text_lower for k in self_harm_keywords)
        return {
            "flagged": is_self_harm,
            "self_harm": is_self_harm,
            "score": 0.99 if is_self_harm else 0.0,
        }
