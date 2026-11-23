# ADR-0008: Invite-only beta enforced on the backend

- **Purpose:** Record the decision, its context and consequences.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR guide](README.md), [AGENTS.md](../../AGENTS.md), [SECURITY_PRIVACY](../SECURITY_PRIVACY.md), [API_CONTRACT](../API_CONTRACT.md)

**ADR status:** Accepted (locked decision; change only via a new ADR and owner approval)

## Context
Free AI quota is shared per project; production on Play is public; strangers could exhaust it. Sign-in is Google plus email one-time code.

## Decision
Sign-up requires a valid invite code or an allowlisted email, enforced on the backend (not the client). The gate stays enforced after Play production release until the owner decides otherwise via a new ADR.

## Consequences
- Protects the shared quota.
- Invite management needs a simple admin path (script in v1).
- Play reviewers need a working test account or invite: plan for it in [RELEASE_PLAYBOOK](../RELEASE_PLAYBOOK.md).
