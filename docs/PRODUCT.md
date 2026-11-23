# Product

- **Purpose:** Define vision, users, the 17 v1 features with acceptance criteria, non-goals and the parked v2 list. Canonical home for feature numbering.
- **Audience:** Owner, agents, reviewers.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ARCHITECTURE](ARCHITECTURE.md), [AI_PIPELINE](AI_PIPELINE.md), [../ROADMAP.md](../ROADMAP.md), [../BACKLOG_V2.md](../BACKLOG_V2.md)

## Vision
A calm, honest AI study partner for students and self-learners who switch between English, Filipino and Bisaya, on mid-range Android phones and unreliable networks. Four input paths (chat, images, files, voice) plus language handling. Built as a long-term Google Play product, not an MVP.

## Target users
Students and self-learners; bilingual users (Taglish, Bisaya-English); mid-range phones (a 4 GB RAM device is a test target); flaky mobile networks.

## v1 features and acceptance criteria
Phase numbers refer to [ROADMAP](../ROADMAP.md).

| # | Feature [v1] | Phase | Acceptance criteria |
|---|---|---|---|
| 1 | Streaming chat and history | 1 | Tokens stream; history lists, searches, renames, deletes; regenerate; edit and resend; Markdown and code render; stop cancels the stream |
| 2 | Camera and gallery images, multi-image | 2 | Up to 4 images per message; resized and location metadata stripped on device; blurry warning before upload |
| 3 | File attachments with extraction | 3 | PDF, DOCX, TXT, CSV accepted; type verified by header; text and tables extracted; clear errors for password-protected, corrupt, unsupported files |
| 4 | Voice transcription, review before send | 4 | Transcript lands in the composer, never auto-sent; low-confidence warning banner |
| 5 | Multilingual replies, code-switching | 4 | Reply matches the user's language and register; explicit override works; tiers shown honestly in app |
| 6 | Study mode | 4 | 3-step hint ladder; check my answer; automated test proves steps 1-2 never reveal the answer |
| 7 | Text-to-speech replies (Android built-in) | 4 | Any reply can be read aloud; Bisaya voice availability labeled honestly |
| 8 | Share to Ruo | 2 | Images, PDFs, text received; copied to sandbox immediately; New chat or Add to recent chat |
| 9 | Document scanner cleanup | 2 | ML Kit Document Scanner output usable as an attachment |
| 10 | Citations over attachments | 3 | Model page/region references render as tappable chips that open the attachment at that spot |
| 11 | Quick action chips | 2-3 | Summarize, Explain, Translate, Solve send a structured intent that selects the prompt template |
| 12 | Feedback buttons | 5 | Thumbs up/down with reason; stored; consented thumbs-down feeds the eval set |
| 13 | Accounts | 1, 5 | Google sign-in and email one-time code; invite-gated; account deletion in app and on web; data export |
| 14 | Usage limits, abuse protection | 1 | Per-user daily quotas and rate limits; clear retry messages; no billing |
| 15 | Safety | 1, 3 | Moderation; calm refusal with support info on self-harm; prompt-injection defense |
| 16 | Offline | 1, 5 | Cached history readable offline (1); drafts and queued messages replay without duplicates (5) |
| 17 | Settings | 1, 5 | Theme, UI language, response language, privacy controls; UI ships in English and Filipino |

Quota starting points live in [SECURITY_PRIVACY](SECURITY_PRIVACY.md).

## Non-goals [v1]
iOS or any other platform; billing; a gateway service; Redis or object storage; features in the v2 list.

## v2 (parked) [v2]
See [../BACKLOG_V2.md](../BACKLOG_V2.md). Not planned during v1.
