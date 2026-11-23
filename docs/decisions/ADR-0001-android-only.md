# ADR-0001: Android only

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [ADR-0002](ADR-0002-rust-uniffi-client-core.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Single solo developer, limited disk, and a target audience on mid-range Android phones.

## Decision
Ruo AI is Android only. No iOS code, now or later. Min SDK 26.

## Consequences
- Smaller scope and one toolchain.
- No cross-platform reuse argument for Rust (see [ADR-0002](ADR-0002-rust-uniffi-client-core.md)).
- Any iOS idea is rejected, not parked.
