import logging
import sys


class StructuredFormatter(logging.Formatter):
    """Simple JSON-like structured formatter without logging sensitive user tokens or message bodies."""

    def format(self, record: logging.LogRecord) -> str:
        request_id = getattr(record, "request_id", "-")
        user_id = getattr(record, "user_id", "-")
        extra_fields = getattr(record, "extra_fields", "")
        base_msg = super().format(record)
        return f"[{record.asctime}] [{record.levelname}] [req:{request_id}] [user:{user_id}] {base_msg} {extra_fields}".strip()


def setup_logging(log_level: str = "INFO") -> None:
    logger = logging.getLogger("ruo")
    logger.setLevel(getattr(logging, log_level.upper(), logging.INFO))

    if not logger.handlers:
        handler = logging.StreamHandler(sys.stdout)
        formatter = StructuredFormatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
        handler.setFormatter(formatter)
        logger.addHandler(handler)


logger = logging.getLogger("ruo")
