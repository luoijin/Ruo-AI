# ADR-0005: $0 infrastructure; Redis and object storage deferred

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [INFRASTRUCTURE](../INFRASTRUCTURE.md), [ADR-0007](ADR-0007-postgres-queue-and-quotas.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Solo developer, no budget for hosting during beta.

## Decision
Run on free tiers only (Oracle Cloud Always Free VM preferred; others UNVERIFIED fallbacks). Local Docker Compose for dev. No Redis and no S3/object storage in v1; Postgres holds data, queue, quotas and temporary upload chunks.

## Consequences
- Raw files purged within 24 hours; storage hygiene is mandatory.
- Free host terms may change: keep providers swappable.
- Add Redis or object storage only after outgrowing one instance, via a new ADR.
