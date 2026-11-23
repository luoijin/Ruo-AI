# Release Playbook

- **Purpose:** Guide Play Console setup, the closed test, production access, staged rollout, store listing, Data Safety and crash reporting.
- **Audience:** Owner.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [SECURITY_PRIVACY](SECURITY_PRIVACY.md), [../ROADMAP.md](../ROADMAP.md), [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md)

Re-check current Play Console requirements when you register (**UNVERIFIED** until then).

## Account setup [v1]
- One-time US$25 registration fee (verified).
- Card: regular credit or debit card; prepaid cards not accepted (from the plan; verify at registration).
- Government ID may be required; account holder must be 18 or older (verify).
- Personal accounts created after November 13, 2023 must run a **closed test with at least 12 opted-in testers for 14 continuous days** before applying for production access (verified). **Internal testing does not count.**

## Timeline [v1]
1. Register early (identity verification can take time).
2. Finish Phase 3 → ship a stable build to a closed track. Recruit more than 12 testers; testers must stay opted in for all 14 days.
3. Keep building through Phases 4-5; push updates to the closed track.
4. After 14 continuous days, apply for production access.
5. Staged rollout: 10% → 50% → 100%.
Production is public: the backend invite gate stays enforced ([ADR-0008](decisions/ADR-0008-invite-only-beta.md)).

## Pre-release checklist [v1]
- [ ] Internal testing on 2-3 real devices including a 4 GB RAM phone
- [ ] Signed release build with Play App Signing; monotonic version codes
- [ ] Crash reporting handling native Rust crashes (Crashlytics or Sentry; free tiers UNVERIFIED)
- [ ] Privacy policy URL; account-deletion URL; Data Safety form
- [ ] Store listing: description, screenshots, feedback contact, honest language-tier statement
- [ ] Free-tier data-use disclosure in policy and in-app
- [ ] Minors/consent decision implemented
- [ ] Accessibility pass done
- [ ] Consider Play in-app updates for urgent fixes

## Data Safety checklist
Declare: account info (email), user content (messages, images, files, audio), diagnostics (crash data), data shared with the AI provider, retention and deletion. Answer from [SECURITY_PRIVACY](SECURITY_PRIVACY.md); do not guess.

## After release
Monitor crashes and quota use; keep staged rollout halting criteria (crash-rate spike, quota exhaustion) written down before rollout.
