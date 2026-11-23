# ADR-0002: Rust core via UniFFI

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [RUST_CORE](../RUST_CORE.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Streaming, reconnects, resumable uploads, encrypted cache and the outbox are the hardest client problems on unreliable networks.

## Decision
A headless Rust core (cargo-ndk + UniFFI) owns SSE streaming and reconnects, resumable uploads, the SQLCipher store, the outbox and validation. It never touches Android lifecycle, content URIs or Compose state. Justification: reliability and skills showcase; not cross-platform reuse.

## Consequences
- Build friction (UniFFI, cargo-ndk, SQLCipher with OpenSSL): mitigated by an early spike and a frozen interface.
- Callbacks arrive on Rust threads; Kotlin must hop to the main dispatcher.
- Clear boundary rules enforced in review.
