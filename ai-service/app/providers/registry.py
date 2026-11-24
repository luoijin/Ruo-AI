from app.core.config import settings
from app.providers.base import ModelProvider
from app.providers.fake import FakeProvider
from app.providers.gemini import GeminiProvider

_fake_provider_instance = FakeProvider()
_gemini_provider_instance = GeminiProvider()


def get_model_provider() -> ModelProvider:
    if settings.MODEL_PROVIDER == "gemini":
        return _gemini_provider_instance
    return _fake_provider_instance
