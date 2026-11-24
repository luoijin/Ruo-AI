from pydantic import BaseModel, EmailStr, Field


class GoogleAuthRequest(BaseModel):
    id_token: str
    invite_code: str | None = None


class EmailOtpRequest(BaseModel):
    email: EmailStr
    invite_code: str | None = None


class EmailOtpVerifyRequest(BaseModel):
    email: EmailStr
    code: str = Field(min_length=6, max_length=6)


class RefreshTokenRequest(BaseModel):
    refresh_token: str


class TokenResponse(BaseModel):
    access_token: str
    refresh_token: str
    token_type: str = "bearer"
    expires_in: int


class UserResponse(BaseModel):
    id: str
    email: str
    ui_lang: str
    preferred_ai_lang: str


class AuthResponse(BaseModel):
    tokens: TokenResponse
    user: UserResponse
