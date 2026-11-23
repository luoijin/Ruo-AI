# Security and Privacy

- **Purpose:** Define the threat model, controls, quotas, retention, disclosures, deletion/export, minors and Play compliance.
- **Audience:** Engineers, agents, owner.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [AI_PIPELINE](AI_PIPELINE.md), [DATA_MODEL](DATA_MODEL.md), [RELEASE_PLAYBOOK](RELEASE_PLAYBOOK.md), [ADR-0008](decisions/ADR-0008-invite-only-beta.md)

This is engineering guidance, not legal advice.

## Threat model (summary) [v1]
| Threat | Control |
|---|---|
| Stolen device reads history | SQLCipher; key wrapped by Android Keystore |
| Traffic interception | TLS only; no cleartext in release builds |
| Provider key leak | Key only on backend; never in app or repo |
| Malicious uploads | Size caps, header-based type check, DOCX zip-bomb protection |
| Prompt injection via documents | Isolation wrapper; no tool surface ([AI_PIPELINE](AI_PIPELINE.md)) |
| Quota exhaustion by strangers | Invite-only enforced on backend ([ADR-0008](decisions/ADR-0008-invite-only-beta.md)); per-user quotas |
| Token theft | Short-lived access tokens; rotating hashed refresh tokens |
| Location leak from photos | EXIF/location stripped on device |

## Controls [v1]
- **Encryption:** SQLCipher local store; TLS in transit.
- **Upload validation:** size cap, header type check, DOCX zip-bomb limits, location metadata stripped on device.
- **Moderation:** moderation check before generation; calm refusal and support information when self-harm content is detected.
- **Rate limits and daily quotas (starting point, tune after verifying real provider limits):** 40 messages, 5 uploads, 15 transcriptions per user per day.
- **Budget alarm:** provider-wide daily budget disables expensive features before limits are hit.

## Free-tier data-use disclosure [v1]
The free Gemini API tier uses submitted content to improve Google's products; the paid tier does not. The privacy policy and an in-app notice must say so plainly, before the user sends content. Moving to a paid tier later removes this. Free-tier eligibility of specific models is **UNVERIFIED**.

## Retention [v1]
Raw uploads: purged after processing or within 24 hours. Persisting: extracted text and embeddings, messages, until the user deletes them or their account. Audio: not retained after transcription (confirm in implementation).

## Deletion and export [v1]
Account deletion in the app and via a public web URL (Google Play requirement), full cascade. Data export is asynchronous (`POST /me/export`). Both tested in Phase 5.

## Minors and consent (OPEN)
Students are the target. Minimum age and consent approach are undecided: see [OPEN_QUESTIONS](OPEN_QUESTIONS.md) Q3. Check the Philippine Data Privacy Act and Google Play policies on apps used by minors before filling the Data Safety form. Decide before Phase 5. Note: a Play developer account holder must be 18 or older.

## Google Play compliance checklist [v1]
- [ ] Privacy policy URL, with free-tier data-use disclosure
- [ ] Data Safety form completed accurately
- [ ] In-app and web account deletion URL
- [ ] Minimal permissions (camera, microphone, optional notifications)
- [ ] Signed release builds, Play App Signing
- [ ] Crash reporting configured, disclosed
- [ ] Minors/consent decision implemented
Full release steps: [RELEASE_PLAYBOOK](RELEASE_PLAYBOOK.md).
