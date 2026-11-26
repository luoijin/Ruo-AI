# Multi-Agent Working Protocol

- **Purpose:** Let five AI agents work in parallel on Ruo AI without colliding. Canonical home for roles, ownership, handoffs and merge rules.
- **Audience:** The owner and all AI agents.
- **Status:** Draft
- **Last updated:** 2026-11-26
- **Related docs:** [../AGENTS.md](../AGENTS.md), [../ROADMAP.md](../ROADMAP.md), [phases/PHASE-1-core-spine.md](phases/PHASE-1-core-spine.md), [PHASE-1-STATUS.md](PHASE-1-STATUS.md)

This adds to [AGENTS.md](../AGENTS.md); it does not replace it. If they conflict, stop and ask the owner.

## Roles and path ownership [v1]
| Role | Owns (may edit) | Read-only elsewhere |
|---|---|---|
| **Lead** (architect, reviewer, docs, CI, merges) | `docs/` (except `docs/status/<other>.md`), `docs/decisions/`, `.github/`, `CHANGELOG.md`, `README.md`, `AGENTS.md`, `skills.md`, root files, `docs/PHASE-1-STATUS.md` | all code |
| **Reviewer** (independent code and PR review) | `docs/status/reviewer.md` only | everything else |
| **Backend** | `ai-service/`, `infra/` | everything else |
| **Rust** | `core-rust/`, the Interface section of `docs/RUST_CORE.md` (proposals only after freeze) | everything else |
| **Android** | `android/` | everything else |

Each agent also owns its own `docs/status/<role>.md`. Editing outside your paths is a **request**, not an edit (see handoffs).

## Worktrees and branches
- One git worktree per agent; never share a working directory.
- Branch names: `agent/<role>/<topic>`. One open PR per agent at a time; small PRs.
- Worktrees: `C:\Github\ruo-ai-wt\lead` (agent/lead/...), `C:\Github\ruo-ai-wt\backend` (agent/backend/...), `C:\Github\ruo-ai-wt\rust` (agent/rust/...), `C:\Github\ruo-ai-wt\android` (agent/android/...), `C:\Github\ruo-ai-wt\review` (agent/review/...). The Reviewer may `git switch --detach origin/<branch>` inside its own review worktree to inspect PR branches; no other agent may switch branches.
- **Session start check:** every agent must confirm `git rev-parse --show-toplevel` is its own worktree and `git branch --show-current` starts with `agent/<role>/`. The worktree path and branch must match the role. Never run `git stash`, `git reset --hard`, `git clean`, or `git switch`/`checkout` to a branch that is not yours in a shared folder. When free-tier allowances are limited, run agents in priority order (Lead first to unblock merges). Commit after every finished step.
- Rebase on `main` before opening a PR. Never force-push `main`. Never rewrite another agent's branch.
- **Only the Lead merges**, squash, after (a) CI is green, (b) the Reviewer's latest `REVIEW VERDICT` comment for the PR's head commit is APPROVE, (c) the Lead's process checks pass, and (d) any gate-level owner approval is recorded. A review is void after new commits. The Reviewer never merges, closes PRs or pushes to other agents' branches.

## Status files (resume point after quota cut-offs)
- `docs/status/<role>.md`, written by that role: current task, branch/PR, what is verified and **where** (host, Postgres, CI, device), what is UNVERIFIED, blockers, and `## Requests to others`.
- `docs/PHASE-1-STATUS.md`, written by the Lead only: current gate, **Owner approvals** (dated, quoted), open PRs, blockers board, next steps.
- At session start every agent reads `PHASE-1-STATUS.md` and all `docs/status/*.md`. Before a session ends, and after every finished step, update your status file and commit.
- Agents act only on gates and tasks listed under **Owner approvals**.

## Handoffs and requests
- Need a change in another role's area? Add `REQUEST to <role>: what, why, acceptance check` in your status file. That role picks it up at its next session start. Do not edit their files.
- **Contract changes** (`docs/API_CONTRACT.md`, `docs/DATA_MODEL.md`, the frozen Rust interface): propose in your status file; Lead drafts the doc change; the owner approves; then Backend/Rust/Android implement in separate PRs. After the interface freeze, any change needs an ADR and owner approval.
- Android integrates against Kotlin interfaces ("ports") with fakes, so it can progress before the bridge is final.

## CHANGELOG
Agents put a one-line entry under "Changelog:" in the PR description. The Lead adds it to `CHANGELOG.md` in the merge. (This avoids merge conflicts on one file.)

## Shared resources: one owner each
| Resource | Owner | Notes |
|---|---|---|
| Physical phone / `adb` | Android | others request device runs |
| Docker, local Postgres (port 5432) | Backend | |
| `infra/.env` with the Gemini key | Backend worktree only | never copy to other worktrees |
| Local Rust builds (`target/`) | Rust | plain-SQLite logic tests only; SQLCipher builds run in CI |
| Android `.so` | CI | downloaded via the script in `scripts/` |
| GitHub Actions config | Lead | |

## Review checklist (every PR)
The Reviewer applies this checklist in depth; the Lead verifies the process items (commit identity, changelog line, status files, approvals).
- [ ] Matches the docs, API contract and the phase doc
- [ ] Tests exist and ran; the PR says where (host, Postgres, CI, device)
- [ ] No secrets, tokens, keys or message content in code, logs or errors
- [ ] User-facing strings in resources (English, Filipino)
- [ ] No v2 features, no new permissions or services without owner approval
- [ ] Claims about versions, quotas, models carry an official source and date or say UNVERIFIED
- [ ] Commit authors are the owner's identity only; no co-author or "Generated by" trailers
- [ ] PR description has the Changelog line; docs updated

## Review protocol
- **Verdict values:** APPROVE, CHANGES REQUESTED, BLOCK.
- **Severities:** BLOCKER, MAJOR, MINOR, NIT.
- Reviewer verdict comments start with `REVIEW VERDICT` and name the head sha reviewed.
- Authors fix findings on the same branch and reply in the PR; a review is void after new commits (wait for a fresh verdict).
- All agents share the owner's GitHub login, so GitHub "Approve" is not used; the `REVIEW VERDICT` comment is the authority.

## Escalate to the owner
Locked-decision or contract changes, requirement conflicts, UNVERIFIED facts blocking work, anything that installs software, adds a permission or paid service, touches the phone destructively, or needs a v2 feature.