# Glossary

- **Purpose:** Define project terms.
- **Audience:** Everyone.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [PRODUCT](PRODUCT.md), [ARCHITECTURE](ARCHITECTURE.md)

| Term | Meaning |
|---|---|
| Outbox | Local queue of unsent messages in SQLCipher; replayed on reconnect without duplicates |
| client_msg_id | Client-generated, time-ordered unique ID that makes sends idempotent |
| Review-before-send | Voice transcripts land in the composer for editing; never auto-sent |
| Hint ladder | Study mode's 3 steps: nudge, scaffold, full walkthrough |
| Check my answer | Study feature: find the first diverging line in the student's work |
| ModelProvider | Backend interface hiding the AI provider (Gemini in v1) |
| Code-switching | Mixing languages in one message (Taglish, Bisaya-English) |
| Taglish | Tagalog/Filipino-English mix |
| Bisaya / Cebuano | Language spoken in Central Visayas and Mindanao |
| Support tiers | Honest language-quality levels shown in the app |
| UniFFI | Tool generating Kotlin bindings for Rust code |
| cargo-ndk | Tool for building Rust libraries for Android |
| SQLCipher | Encrypted SQLite |
| SSE | Server-Sent Events: one-way streaming over HTTP |
| Hybrid retrieval | Vector plus keyword search over document chunks |
| Citation chip | Tappable reference that opens an attachment at a page |
| Injection-isolation wrapper | Code that marks untrusted document text as data before prompting |
| Invite gate | Backend rule that only invited or allowlisted users may sign up |
| Closed test | Play testing track requiring 12+ testers for 14 days |
| MVI | Unidirectional UI pattern: State, Intent, Effect |
| ADR | Architecture Decision Record |
