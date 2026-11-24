from abc import ABC, abstractmethod

from app.core.logging import logger


class EmailSender(ABC):
    @abstractmethod
    async def send_otp_email(self, to_email: str, otp_code: str) -> None:
        """Send a one-time verification code to the target email address."""
        pass


class ConsoleEmailSender(EmailSender):
    async def send_otp_email(self, to_email: str, otp_code: str) -> None:
        logger.info(f"[DEV EMAIL SENDER] To: {to_email} | One-time code: {otp_code} (Valid for 10 minutes)")


def get_email_sender() -> EmailSender:
    # Console / log sender for development and tests; real transactional provider in Phase 5
    return ConsoleEmailSender()
