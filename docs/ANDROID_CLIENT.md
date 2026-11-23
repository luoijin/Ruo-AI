# Android Client

- **Purpose:** Specify the client's modules, patterns, navigation, composer, offline behavior, permissions, accessibility and visual direction.
- **Audience:** Android engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [RUST_CORE](RUST_CORE.md), [ARCHITECTURE](ARCHITECTURE.md), [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md), [../skills.md](../skills.md)

## Baseline [v1]
Min SDK 26 (Android 8.0). Target and compile SDK: the current stable level supported by the Compose BOM in use (**UNVERIFIED**, see [VERIFY_BEFORE_BUILD](VERIFY_BEFORE_BUILD.md)).

## Modules [v1]
`:app` (application, activities, navigation) · `:core:ui` (design tokens, Markdown and code renderer, attachment preview, citation chips) · `:core:rustbridge` (UniFFI wrapper, Keystore key handling, Flow adapters) · `:feature:chat` · `:feature:camera` · `:feature:auth` · `:feature:settings`. Tree: [FOLDER_STRUCTURE](FOLDER_STRUCTURE.md).

## Pattern: unidirectional MVI [v1]
Each screen has `UiState` (immutable), a sealed `UiIntent`, and one-off `UiEffect` (snackbars, navigation). ViewModels collect Kotlin Flows bridged from the Rust core. Hilt for DI (verify it still suits).

## Navigation (type-safe routes) [v1]
`Auth` · `Chat(conversationId?)` · `Scanner` · `Settings` · `AttachmentViewer(attachmentId, page)`.

## Composer tray [v1]
Horizontal row of thumbnails or file cards, each with a remove button; page-number badges or drag handles for multi-image messages; a progress ring: Uploading → Extracting → Ready or Failed. Quick action chips (Summarize, Explain, Translate, Solve) appear on attachments.

## Offline behavior [v1]
- Drafts save per conversation about every 500 ms (via Rust).
- A message sent offline is stored as Queued with a "waiting for network" badge.
- A WorkManager job with a network constraint triggers the Rust outbox flush on reconnect.
- A persistent offline banner shows while disconnected.

## Permissions (just in time) [v1]
Camera when the user taps Camera; microphone when they tap Mic; optional notifications (Android 13+) for long extractions. No broad storage permissions: use the system Photo Picker and Storage Access Framework. Adding any permission needs owner approval.

## Accessibility [v1]
48 dp minimum touch targets; content descriptions on icon buttons and attachment previews; polite live-region announcement when a reply finishes streaming; font scaling to 200% without clipping; WCAG AA contrast.

## Visual direction (starting point, not locked) [v1]
Calm, warm, academic-notebook feel for long study sessions; no neon gradients.
| Token | Light | Dark |
|---|---|---|
| background | #FAF9F6 | #111315 |
| surface | #FFFFFF | #1A1D21 |
| surface variant | #F1EFEA | #252930 |
| primary | #1E4D4B | #5EEAD4 |
| accent (study hints) | #D97706 | #FBBF24 |
| error | #B91C1C | not specified |

Type: Plus Jakarta Sans (headings), Inter (body), JetBrains Mono (code, math). Check each font's license and Android bundling before committing. Motion: subtle streaming cursor, collapsible "reading your PDF" indicator, light haptic tick on record start/stop, swipe on bubbles to reply or edit.

## Kotlin vs Rust
Kotlin owns: Compose UI, CameraX, ML Kit scanner, file and photo pickers, audio record/playback, permissions, share intents, image resize and metadata strip. Rust owns: encrypted DB, outbox, SSE and reconnects, chunked uploader and hashing, client validation. The Rust callback threads are not the main thread: the wrapper hops to the main dispatcher before touching UI state ([RUST_CORE](RUST_CORE.md)).

## Mid-range phone efficiency [v1]
Compress images off the main thread; record audio to a file, not memory; cancel in-flight streams when the user leaves the screen or taps stop; cap the thumbnail cache (about 100 MB).

## Strings [v1]
All user-facing text in resource files; English (`values/`) and Filipino (`values-fil/`) first. UI language and response language are separate settings.
