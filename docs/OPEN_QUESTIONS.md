# Open Questions

- **Purpose:** Track open decisions with recommended defaults and owners.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md), [PRODUCT](PRODUCT.md)

| # | Question | Recommended default | Owner | Blocks |
|---|---|---|---|---|
| Q1 | Beta hosting | Try Oracle Always Free first; fall back to free PaaS + free Postgres | Owner | Phase 3 closed test |
| Q2 | Math rendering | Markdown with monospace math in Phase 1; add a scoped renderer later if needed | Owner | none |
| Q3 | Minimum user age and consent approach (students are the target) | Decide before Phase 5; check Philippine Data Privacy Act and Play minors policies | Owner | Phase 5 |
| Q4 | Hilt as DI | Use Hilt unless it conflicts with the verified toolchain | Owner | Phase 1 |
| Q5 | Which Gemini model for chat, fallback, transcription, embeddings | Verify in AI Studio; start with gemini-4-hydrogen only if free-tier eligible (UNVERIFIED) | Owner | **Phase 1** |
| Q6 | Email-sending service for one-time codes | A free-tier transactional provider or SMTP (UNVERIFIED) | Owner | **Phase 1** |
| Q7 | Invite management in v1 | A CLI/script that creates invite codes and allowlist entries | Owner | Phase 1 |
| Q8 | Repository license | Choose before making the repo public (default: all rights reserved) | Owner | Portfolio |
| Q9 | Android application ID / package name | Pick once; it cannot change after Play publication | Owner | **Phase 1** |
| Q10 | Cancel mechanism in the Rust interface | Add a cancel handle or `cancel_stream(message_id)` in the Phase 1 freeze | Owner | **Phase 1** |
| Q11 | Plan-vs-prompt note: email sign-in was an open default in the plan | Treated as **locked** per the project instructions; recorded in ADR-0008 | Owner | none |
| Q12 | Source plan file is a PDF with some text mangling (e.g. language tag in SSE example) | Place the text version at `docs/source/ruo-ai-v1-final-plan.md`; this scaffold uses the corrected `tl-PH` | Owner | none |
