from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, Field, field_validator


class ConversationCreate(BaseModel):
    conversation_id: str = Field(..., min_length=36, max_length=36)
    title: str = Field(default="Bagong Pag-uusap", min_length=1, max_length=255)
    active_language: str = Field(default="en")
    study_mode_default: bool = Field(default=False)

    @field_validator("conversation_id")
    @classmethod
    def validate_conversation_id(cls, value: str) -> str:
        return str(UUID(value))


class ConversationUpdate(BaseModel):
    title: str | None = Field(default=None, min_length=1, max_length=255)
    active_language: str | None = None
    study_mode_default: bool | None = None


class ConversationResponse(BaseModel):
    id: str
    title: str
    active_language: str
    study_mode_default: bool
    created_at: datetime
    updated_at: datetime
    deleted_at: datetime | None = None


class MessageItem(BaseModel):
    id: str
    conversation_id: str
    client_msg_id: str
    parent_message_id: str | None = None
    role: str
    content_md: str
    citations_json: str | None = None
    study_step: int | None = None
    language: str | None = None
    model_used: str | None = None
    status: str
    created_at: datetime


class SendMessageRequest(BaseModel):
    client_msg_id: str = Field(..., description="Unique client-generated message ID")
    content: str = Field(..., min_length=1)
    parent_message_id: str | None = None
    attachment_ids: list[str] = Field(default_factory=list)
    intent: str | None = None
    study_mode: bool = False
