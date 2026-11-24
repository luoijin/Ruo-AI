from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=(".env", "infra/.env", "../infra/.env"),
        env_file_encoding="utf-8",
        extra="ignore",
    )

    ENVIRONMENT: str = Field(default="development")
    LOG_LEVEL: str = Field(default="INFO")
    HOST: str = Field(default="0.0.0.0")
    PORT: int = Field(default=8000)

    # Database
    DATABASE_URL: str = Field(default="sqlite+aiosqlite:///./ruo_dev.db")
    SYNC_DATABASE_URL: str = Field(default="sqlite:///./ruo_dev.db")

    # Auth & Tokens
    JWT_SECRET_KEY: str = Field(default="dev-insecure-secret-key-change-in-production-at-least-32-chars")
    JWT_ALGORITHM: str = Field(default="HS256")
    JWT_ACCESS_TOKEN_EXPIRE_MINUTES: int = Field(default=15)
    REFRESH_TOKEN_EXPIRE_DAYS: int = Field(default=30)
    EMAIL_OTP_EXPIRE_MINUTES: int = Field(default=10)
    EMAIL_OTP_MAX_ATTEMPTS: int = Field(default=5)

    # Google Auth (ID token client verification)
    GOOGLE_CLIENT_ID: str = Field(default="ruo-google-client-id-placeholder")

    # Providers
    MODEL_PROVIDER: str = Field(default="fake")  # "fake" or "gemini"
    GEMINI_API_KEY: str = Field(default="")
    GEMINI_CHAT_MODEL: str = Field(default="gemini-4-hydrogen")  # verified model name
    GEMINI_FALLBACK_MODEL: str = Field(default="gemini-1.5-flash")  # configurable fallback

    # Email
    EMAIL_FROM: str = Field(default="no-reply@ruo.ai")

    # Quotas & Rate Limits
    DAILY_MESSAGE_QUOTA: int = Field(default=40)
    HOURLY_RATE_LIMIT: int = Field(default=100)

    @field_validator("*", mode="before")
    @classmethod
    def empty_str_to_default(cls, v, info):
        # Prevent empty strings from template .env files from overriding sensible defaults
        if isinstance(v, str) and v.strip() == "":
            field = cls.model_fields.get(info.field_name)
            if field is not None and field.default is not ...:
                return field.default
        return v


settings = Settings()
