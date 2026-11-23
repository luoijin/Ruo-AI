# Contributing

- **Purpose:** Branching, commits, PR checklist and code style.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [AGENTS.md](AGENTS.md), [CHANGELOG.md](CHANGELOG.md)

## Branches
`main` is always releasable. Work on `feat/<phase>-<slug>`, `fix/<slug>`, `docs/<slug>`, `chore/<slug>`.

## Commits
Conventional Commits: `feat(chat): stream deltas`, `fix(rust): reconnect backoff`, `docs: ...`. Scopes: `android`, `rust`, `ai-service`, `infra`, `docs`.

## PR checklist
- [ ] Small and single-purpose
- [ ] Tests added/updated and passing
- [ ] Docs updated (and ADR if significant)
- [ ] [CHANGELOG](CHANGELOG.md) `[Unreleased]` updated
- [ ] No secrets, no hard-coded user-facing strings
- [ ] Phase and feature number named in the description

## Code style
| Language | Style | Tooling (UNVERIFIED choices; confirm in Phase 1) |
|---|---|---|
| Kotlin | Official Kotlin style; MVI per [ANDROID_CLIENT](docs/ANDROID_CLIENT.md) | ktlint or detekt |
| Rust | `rustfmt` defaults; `clippy` clean | cargo fmt, cargo clippy |
| Python | Type-hinted, async-first | ruff or equivalent |
| Markdown | Header block on every doc; one canonical home per fact | none |
