# ADR-0007: Postgres-based job queue, rate limits and quotas

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [DATA_MODEL](../DATA_MODEL.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Redis is deferred ([ADR-0005](ADR-0005-zero-cost-infrastructure.md)) but extraction jobs and quotas are needed.

## Decision
An in-process worker claims jobs from a Postgres table with row locking and a heartbeat to recover stale jobs after restarts. Rate limits and daily quotas use atomic insert-or-update on `usage_logs`.

## Consequences
- Simple, one datastore.
- Throughput is limited to one instance; fine for an invite-only beta.
- Revisit if the worker contends with chat streaming for CPU.
