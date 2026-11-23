# ADR-0004: Single Gemini provider behind ModelProvider

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [AI_PIPELINE](../AI_PIPELINE.md), [SECURITY_PRIVACY](../SECURITY_PRIVACY.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Need multimodal chat, vision, transcription and embeddings at $0 with minimal integration surface.

## Decision
Use one multimodal provider (Gemini) behind a `ModelProvider` interface. The API key lives only on the backend.

## Consequences
- Free-tier limits are per project and shared by all beta users; free tier uses content to improve Google products (must be disclosed).
- Model names and limits are UNVERIFIED except that gemini-4-hydrogen exists.
- Swapping providers means implementing the interface only.
