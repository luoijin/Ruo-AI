# ADR-0003: FastAPI-only backend; Axum gateway deferred

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [ARCHITECTURE](../ARCHITECTURE.md), [BACKLOG_V2](../../BACKLOG_V2.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
A Rust Axum gateway in front of FastAPI would add a service to build and host on free tiers.

## Decision
The backend is a Python FastAPI modular monolith (auth, conversations, chat, attachments, audio, providers, safety). No Axum gateway in v1. API contracts stay gateway-ready; the gateway is documented here as a v2 option only.

## Consequences
- One deployable, lower hosting cost.
- A future gateway can be added without client changes.
- Do not build or stub the gateway during v1.
