"""Support keyset sync and idempotent stream completion.

Revision ID: 0002_sync_stream
Revises: 0001_initial
Create Date: 2026-11-25
"""

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

revision: str = "0002_sync_stream"
down_revision: str | None = "0001_initial"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column(
        "messages",
        sa.Column("status", sa.String(length=20), nullable=False, server_default="complete"),
    )
    op.add_column("messages", sa.Column("updated_at", sa.DateTime(), nullable=True))
    op.execute("UPDATE messages SET updated_at = created_at WHERE updated_at IS NULL")
    op.alter_column("messages", "updated_at", nullable=False)

    op.drop_index("idx_conv_user_updated", table_name="conversations")
    op.create_index("idx_conv_user_updated", "conversations", ["user_id", "updated_at", "id"], unique=False)
    op.drop_index("idx_msg_conv_created", table_name="messages")
    op.create_index("idx_msg_conv_created", "messages", ["conversation_id", "created_at", "id"], unique=False)


def downgrade() -> None:
    op.drop_index("idx_msg_conv_created", table_name="messages")
    op.create_index("idx_msg_conv_created", "messages", ["conversation_id", "created_at"], unique=False)
    op.drop_index("idx_conv_user_updated", table_name="conversations")
    op.create_index("idx_conv_user_updated", "conversations", ["user_id", "updated_at"], unique=False)
    op.drop_column("messages", "updated_at")
    op.drop_column("messages", "status")
