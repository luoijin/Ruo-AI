# Infrastructure

- **Purpose:** Describe the $0 hosting options, Docker Compose layout, environments, CI/CD, secrets, observability, backups and cost controls.
- **Audience:** Owner and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ADR-0005](decisions/ADR-0005-zero-cost-infrastructure.md), [DEVELOPMENT_SETUP](DEVELOPMENT_SETUP.md), [SECURITY_PRIVACY](SECURITY_PRIVACY.md)

Target: **$0 per month** [v1]. All hosting terms below are **UNVERIFIED**: check card requirement, idle sleep, RAM/CPU and storage before choosing.

## Hosting options for the beta
| Option | Pros | Cons / unknowns |
|---|---|---|
| **Oracle Cloud Always Free VM** (preferred, if obtainable) | No cold starts; enough RAM for parsing | Availability and signup requirements UNVERIFIED |
| Koyeb or Render (free PaaS) | Easy deploy | Terms changing; idle sleep; CPU too small to parse and stream together (a 0.1 vCPU instance is too small) |
| Neon or Supabase (free Postgres) | Managed DB | Storage caps, pause behavior UNVERIFIED |

Fallback order: Oracle VM → free PaaS + free Postgres. Keep the provider interface swappable.

## Docker Compose (local dev) [v1]
Services: `backend` (FastAPI) and `db` (Postgres with pgvector). File: `infra/docker-compose.yml`; variables in `infra/.env.example` (names only). The database port is bound to localhost only (`127.0.0.1:5432:5432`). Note: running `alembic upgrade head` automatically on container startup is strictly a dev convenience in `docker-compose.yml`; in production/staging environments, database migrations must be run as an explicit, separate release step before starting application processes. The phone connects to the laptop over LAN or USB reverse proxy ([DEVELOPMENT_SETUP](DEVELOPMENT_SETUP.md)). No emulator.

## Environments [v1]
| Env | Where | Notes |
|---|---|---|
| dev | Laptop Docker Compose | Fake or real provider key in local `.env` |
| beta | Free host | Invite-only; production-like config |
| prod | Same host class | Public Play release; invite gate stays enforced |

## Uploads on ephemeral disks [v1]
Chunk data and state live temporarily in Postgres (free hosts restart and wipe disk). Purge raw files after processing or within 24 hours. If too heavy, fall back to a single retryable upload under the size cap.

## CI/CD [v1]
GitHub Actions free tier: Rust tests, backend tests (pytest with Postgres service), Android build. Workflows live in repo-root `.github/workflows/`. Deployment is manual or script-based in v1.

## Secrets [v1]
Environment variables on the host; `.env` gitignored; GitHub Actions secrets for CI. Keys: Gemini API key, JWT signing secret, email-sending credentials (provider UNVERIFIED), Google OAuth client ID. Never in app or repo. Rotation procedure documented in `infra/deploy/`.

## Observability [v1]
Structured logs with `request_id` (never log message content or tokens); health endpoint; crash reporting on the client (Firebase Crashlytics or Sentry, free tiers UNVERIFIED; must handle native Rust crashes).

## Backups [v1]
Free-tier backup guarantees are UNVERIFIED. Run a scheduled `pg_dump` to a location you control, and test a restore once before the closed test.

## Cost-control mechanisms [v1]
Per-user quotas · invite-only access · provider-wide daily budget alarm · lighter-model fallback · file-size caps · raw file purge · storage hygiene ([DATA_MODEL](DATA_MODEL.md)).

## Deferred [v2 / when outgrown]
Redis and object storage: add only when one instance or storage limits are outgrown, via a new ADR.
