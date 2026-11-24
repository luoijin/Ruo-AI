from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Protocol


@dataclass
class ChatChunk:
    text: str
    finish_reason: str | None = None
    input_tokens: int = 0
    output_tokens: int = 0


@dataclass
class ChatRequest:
    messages: list[dict[str, str]]
    model: str | None = None
    system_instruction: str | None = None
    temperature: float = 0.7


class ModelProvider(Protocol):
    async def stream_chat(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        """Stream response tokens as chunks."""
        ...

    async def moderate(self, text: str) -> dict[str, bool | float]:
        """Basic moderation check; returns flags like is_harmful, self_harm."""
        ...
