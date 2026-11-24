import asyncio
from collections.abc import AsyncIterator

from app.providers.base import ChatChunk, ChatRequest, ModelProvider


class FakeProvider(ModelProvider):
    """Deterministic streaming provider with Taglish sample for offline tests and development."""

    def __init__(self, sample_response: str | None = None, delay_sec: float = 0.01):
        self.sample_response = sample_response or (
            "Kumusta! Handa akong tulungan ka sa iyong pag-aaral. "
            "Una, alamin natin kung ano ang target concept na kailangan mong lutasin. "
            "May tanong ka ba tungkol sa math, science, o Filipino?"
        )
        self.delay_sec = delay_sec

    async def stream_chat(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        # Split sample response into small token-like pieces
        words = self.sample_response.split(" ")
        for i, word in enumerate(words):
            if self.delay_sec > 0:
                await asyncio.sleep(self.delay_sec)
            prefix = "" if i == 0 else " "
            yield ChatChunk(text=f"{prefix}{word}")

        # Final chunk with finish reason and token stats
        yield ChatChunk(
            text="",
            finish_reason="stop",
            input_tokens=len(request.messages) * 15,
            output_tokens=len(words) * 2,
        )

    async def moderate(self, text: str) -> dict[str, bool | float]:
        text_lower = text.lower()
        # Basic self-harm and harm keywords detection
        self_harm_keywords = ["magpakamatay", "suicide", "kill myself", "end my life", "laslas"]
        is_self_harm = any(k in text_lower for k in self_harm_keywords)
        return {
            "flagged": is_self_harm,
            "self_harm": is_self_harm,
            "score": 0.99 if is_self_harm else 0.0,
        }
