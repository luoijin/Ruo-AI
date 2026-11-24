from pydantic import Field, field_validator, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

DEV_JWT_SECRET_PLACEHOLDER = "dev-insecure-secret-key-change-in-production-at-least-32-chars"
DEV_GOOGLE_CLIENT_ID_PLACEHOLDER = "ruo-google-client-id-placeholder"


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
    JWT_SECRET_KEY: str = Field(default=DEV_JWT_SECRET_PLACEHOLDER)
    JWT_ALGORITHM: str = Field(default="HS256")
    JWT_ACCESS_TOKEN_EXPIRE_MINUTES: int = Field(default=15)
    REFRESH_TOKEN_EXPIRE_DAYS: int = Field(default=30)
    EMAIL_OTP_EXPIRE_MINUTES: int = Field(default=10)
    EMAIL_OTP_MAX_ATTEMPTS: int = Field(default=5)

    # Google Auth (ID token client verification)
    GOOGLE_CLIENT_ID: str = Field(default=DEV_GOOGLE_CLIENT_ID_PLACEHOLDER)

    # Providers
    MODEL_PROVIDER: str = Field(default="fake")  # "fake" or "gemini"
    GEMINI_API_KEY: str = Field(default="")
    GEMINI_CHAT_MODEL: str = Field(default="gemini-4-hydrogen")  # verified model name
    GEMINI_FALLBACK_MODEL: str | None = Field(default=None)  # unconfigured (UNVERIFIED)

    # Email
    EMAIL_FROM: str = Field(default="no-reply@ruo.ai")

    # Quotas & Rate Limits
    DAILY_MESSAGE_QUOTA: int = Field(default=40)

    @field_validator("*", mode="before")
    @classmethod
    def empty_str_to_default(cls, v, info):
        # Do not allow empty strings for security-sensitive keys to silently fall back to dev placeholders
        if info.field_name in ("JWT_SECRET_KEY", "GOOGLE_CLIENT_ID", "GEMINI_API_KEY", "GEMINI_FALLBACK_MODEL"):
            return v
        if isinstance(v, str) and v.strip() == "":
            field = cls.model_fields.get(info.field_name)
            if field is not None and field.default is not ...:
                return field.default
        return v

    @model_validator(mode="after")
    def validate_production_secrets(self) -> "Settings":
        if self.ENVIRONMENT != "development":
            if (
                not self.JWT_SECRET_KEY
                or not isinstance(self.JWT_SECRET_KEY, str)
                or self.JWT_SECRET_KEY.strip() == ""
                or self.JWT_SECRET_KEY == DEV_JWT_SECRET_PLACEHOLDER
            ):
                raise ValueError("JWT_SECRET_KEY must be set to a secure secret outside development.")
            if len(self.JWT_SECRET_KEY) < 32:
                raise ValueError("JWT_SECRET_KEY must be at least 32 characters long.")

            if (
                not self.GOOGLE_CLIENT_ID
                or not isinstance(self.GOOGLE_CLIENT_ID, str)
                or self.GOOGLE_CLIENT_ID.strip() == ""
                or self.GOOGLE_CLIENT_ID == DEV_GOOGLE_CLIENT_ID_PLACEHOLDER
            ):
                raise ValueError("GOOGLE_CLIENT_ID must be configured outside development.")
        else:
            # In development, if empty in a template .env, use dev placeholders
            if not self.JWT_SECRET_KEY or (isinstance(self.JWT_SECRET_KEY, str) and self.JWT_SECRET_KEY.strip() == ""):
                self.JWT_SECRET_KEY = DEV_JWT_SECRET_PLACEHOLDER
            if not self.GOOGLE_CLIENT_ID or (
                isinstance(self.GOOGLE_CLIENT_ID, str) and self.GOOGLE_CLIENT_ID.strip() == ""
            ):
                self.GOOGLE_CLIENT_ID = DEV_GOOGLE_CLIENT_ID_PLACEHOLDER
        return self


settings = Settings()
