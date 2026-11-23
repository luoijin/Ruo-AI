# Phase 3: Documents and citations

- **Purpose:** Plan and gate Phase 3: Documents and citations.
- **Audience:** Engineers and agents.
- **Status:** Draft
- **Last updated:** 2026-11-23
- **Related docs:** [ROADMAP](../../ROADMAP.md), [PRODUCT](../PRODUCT.md), [AGENTS.md](../../AGENTS.md), [skills.md](../../skills.md)

## Goal
Extract and reason over PDF, DOCX, TXT, CSV with page citations and injection defenses. Effort: medium. **Start the Play closed test after this phase.**

## Features covered [v1]
Features 3 (file attachments), 10 (citations), 11 (chips on documents), 15 (injection defense).

## Ordered tasks
1. Tables: extracted_content, document_chunks, processing_jobs. Confirm embedding model and dimension first. [v1]
2. Postgres-backed job worker with row locking and heartbeat. [v1]
3. Extraction: PyMuPDF, pdfplumber, python-docx; CSV/TXT; vision OCR only for scanned pages. [v1]
4. Caps: 15 MB, 50 PDF pages, smaller for DOCX/TXT; zip-bomb protection. [v1]
5. Short vs long: full text under about 12,000 tokens; otherwise chunk (about 500 tokens, overlap), embed, hybrid retrieval. [v1]
6. Injection-isolation wrapper on all document/OCR text. [v1]
7. Citation events and tappable chips; AttachmentViewer route opens at the page. [v1]
8. Attachment status SSE and tray states; failure messages (password-protected, corrupt, unsupported). [v1]
9. Eval fixtures: scans, multi-column PDFs, tables, injection documents. [v1]
10. **Prepare and start the Play closed test** ([RELEASE_PLAYBOOK](../RELEASE_PLAYBOOK.md)).

## Dependencies
Phase 2; embedding model and dimension verified; Play account registered.

## Acceptance criteria
- Ask about a 50-page PDF and tap a citation to open the right page.
- Follow-ups do not re-parse the file.
- Hostile-instruction documents do not change model behavior.
- Raw files purged within the retention limit.

## Test requirements
Backend tests with real PDF/DOCX fixtures; type-spoofing and zip-bomb rejection; injection delimiter escaping; retrieval tests; load test for concurrent SSE plus parsing.

## Agent notes (pitfalls)
- Never bypass the isolation wrapper.
- Do not parse on the request thread; use the worker.
- Free-host CPU is small: measure parsing latency early.
- Include a test account/invite for Play reviewers.

## Definition of done
- [ ] Acceptance criteria met and demonstrated on a physical phone
- [ ] Tests listed above passing in CI
- [ ] Docs, API_CONTRACT/DATA_MODEL (if changed) and CHANGELOG updated
- [ ] No unverified fact stated as fact; VERIFY_BEFORE_BUILD statuses updated
- [ ] Security checklist in [skills.md](../../skills.md) applied
