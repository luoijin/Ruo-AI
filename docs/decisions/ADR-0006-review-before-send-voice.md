# ADR-0006: Review-before-send voice

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [ARCHITECTURE](../ARCHITECTURE.md), [AI_PIPELINE](../AI_PIPELINE.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Code-switched speech (Taglish, Bisaya-English) produces transcription errors; sending a wrong transcript wastes quota and misleads the answer.

## Decision
Transcripts are placed in the composer for review and editing. Never auto-send. Low-confidence transcripts show a warning banner.

## Consequences
- One extra tap per voice message.
- Fewer bad prompts; honest handling of Tier 2 languages.
- Any 'hands-free send' idea is a v2 question.
