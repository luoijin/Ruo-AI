from datetime import date
from typing import TYPE_CHECKING

from sqlalchemy import BigInteger, Date, ForeignKey, Integer, String, UniqueConstraint
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.core.database import Base
from app.models.common import generate_uuid

if TYPE_CHECKING:
    from app.auth.models import User


class UsageLog(Base):
    __tablename__ = "usage_logs"

    id: Mapped[str] = mapped_column(String(36), primary_key=True, default=generate_uuid)
    user_id: Mapped[str] = mapped_column(String(36), ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    usage_date: Mapped[date] = mapped_column(Date, nullable=False)
    request_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    tokens_in: Mapped[int] = mapped_column(BigInteger, default=0, nullable=False)
    tokens_out: Mapped[int] = mapped_column(BigInteger, default=0, nullable=False)
    upload_bytes: Mapped[int] = mapped_column(BigInteger, default=0, nullable=False)

    user: Mapped["User"] = relationship("User", back_populates="usage_logs")

    __table_args__ = (UniqueConstraint("user_id", "usage_date", name="uq_user_usage_date"),)
