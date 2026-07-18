# AGENTS.md

## Project Summary

- Project name: `Inkform`
- Goal: turn a handwriting sample into a usable downloadable font
- Product type: consumer creative tool for OpenAI Build Week
- Track target: `Apps for Your Life`

## Current Scope

- Vercel Hobby frontend only for hosting
- Rust-first architecture with a shared core crate and a WASM-facing wrapper crate
- Guaranteed v1 language coverage: Latin extended
- Anonymous usage only in v1
- Freeform handwriting upload, local processing pipeline, preview, and export flow

## Non-Goals

- Full Unicode support in the initial submission
- Account system, billing, or collaboration features
- Separate always-on backend service for v1
- Long-term persistence of user handwriting artifacts

## Architecture Decisions

- Keep the deterministic generation logic in `crates/inkform-core`
- Keep browser integration concerns in `crates/inkform-wasm`
- Treat the frontend as a product shell over the Rust engine
- Favor local browser processing to avoid Vercel Hobby backend limits
- Keep script coverage data-driven so new script packs can be added later
- Do not require a guided handwriting sheet in the primary product flow
- Do not use an embedded reference font in generation
- Use the uploaded handwriting image to derive style signals; use the in-project glyph grammar to
  synthesize complete, legible character topology for freeform samples
- Cursive samples additionally drive grammar centerline rounding and open-stroke terminal taper;
  do not add disconnected pseudo-ligature strokes without contextual shaping support
- The optional sample transcript describes text already visible in the uploaded photo. For a
  single line, Inkform isolates terminal punctuation, partitions the word at low-ink valleys using
  transcript-derived character-width targets, and accepts anchors only when every extracted region
  passes shape validation. It must fall back to style synthesis when those checks fail
- The global style profile must use complete source strokes. When transcript-aligned centerlines
  pass safety checks, re-stroke those exact anchor paths; unseen characters continue through the
  shared controlled-stroke grammar
- Centerline extraction uses bounded Zhang-Suen thinning on an adaptive-resolution bitmap. Reuse
  only one- or two-trajectory anchors with bounded complexity; connected cursive photos that split
  into many branches must inform style synthesis rather than be replayed as illegible glyphs
- For strongly cursive samples, only glyphs with naturally open terminals receive a thin exit
  stroke that overlaps the following glyph; closed bowls and counters prioritize legibility over
  a synthetic connector. Keep this as one shared contextual connection strategy, not a full
  baseline underline or disconnected decorative ligatures
- Literal transcript anchors must pass centerline continuity and character-topology checks before
  replay. Closed counters, dotted letters, crossed letters, and multi-stroke letters otherwise
  fall back to the legible grammar while retaining the upload's global style
- When transcript alignment succeeds, use the aligned character width as the primary advance-width
  signal. Blend it with the bounded script default only to keep unseen or noisy samples legible;
  do not use a fixed Latin tracking value for every upload
- Normalize generated contours to their configured left bearing before calculating a transcript-
  aligned advance. Use a small bounded trailing allowance so loops remain distinct without
  reintroducing generic side margins
- Strong-cursive grammar uses connected or overlapping strokes for `k`, `m`, `n`, and `r`, gives
  `f` both ascender and descender loops, and keeps synthetic exit strokes off `m`, `n`, and `r`;
  capital `I` includes top and bottom crossbars. Avoid retracing a stem in one outline because it
  pinches the generated glyph
- Strong-cursive capital `H` preserves an entering left swash, while cursive `e` uses one compact
  continuous loop with no synthetic exit stroke; this avoids the unreadable crossed-counter shape
- A real `Hello!` sample can produce overlapping connected-stroke regions plus separate
  punctuation strokes. Preserve the transcript-aligned segmentation confidence checks; never map
  raw components monotonically to characters

## Rust Coding Constraints

- No `unsafe`
- No `unwrap`
- Avoid unnecessary `clone`s
- Prefer explicit error handling with typed error enums
- Run clippy before every commit
- Maintain a real test suite, not just smoke tests
- Do not push commits unless the user explicitly requests a push

## Deployment Constraints

- Frontend target: Vercel Hobby
- No required separate backend host in v1
- Keep the runtime model compatible with browser execution
- Avoid designs that depend on long-running Vercel functions

## Frontend Workflow Notes

- `frontend` `npm run dev` now routes through `scripts/dev-frontend.sh`, which rebuilds WASM, tries to stop a stale repo-local Next dev server on port `3000`, and then starts a clean server on `3000`
- `frontend` runs `npm run wasm:build` automatically before `npm run build`
- `scripts/build-wasm.sh` copies the generated JS wrapper into
  `frontend/src/lib/generated` for Turbopack to bundle; loading the raw public JS
  via browser `import()` can fail in Next development. Keep the `.wasm` binary in
  `frontend/public/wasm` and initialize the bundled wrapper with its public URL.
- The development-only `/api/dev/local-*` routes require
  `INKFORM_DEV_SAMPLE_DIRECTORY` in `frontend/.env.local`; use
  `frontend/.env.example` as the non-machine-specific template.
- `frontend/vercel.json` assumes Vercel's project Root Directory is `frontend`.
  The frontend build consumes committed WASM delivery artifacts because Vercel's
  root-directory isolation prevents it from compiling the parent Rust workspace.
  CI rebuilds those artifacts and fails if they are stale.
- If the browser starts showing a generic placeholder preview again, first verify that the WASM bundle was rebuilt from current Rust sources
- In restricted sandboxes, `wasm-pack` may fail because `wasm-bindgen` install/cache paths are not writable or network access is unavailable; a local machine run is the source of truth for browser-side validation
- The current preview fallback path is `svg-v3`, tuned to visually match the generated TTF when a browser font load genuinely fails
- Browser-compatible fonts require `cmap` format-4 binary-search fields using the format's
  2-byte multiplier, not the sfnt table-directory 16-byte multiplier. Keep the regression test
  for this invariant; Zen directly loads the resulting generated TTF

## Dependency Safety Rules

- Do not install new npm packages blindly
- Verify direct frontend dependencies against primary sources before install when versions change
- Prefer exact pinned versions over broad ranges
- Prefer first-party or widely established packages unless there is a strong reason otherwise
- Use `npm install --ignore-scripts` first when introducing or refreshing frontend dependencies
- Inspect lockfile and dependency tree before normalizing the install flow
- Treat supply-chain risk as a product risk, not just a tooling detail

## Context Maintenance

- Periodically check whether new decisions, risks, workflow rules, or environment notes should be added to `AGENTS.md`
- Treat `AGENTS.md` as the durable project memory, not a one-time setup file

## Submission Checklist

- [ ] Runnable app deployed to a public URL
- [ ] Public repo URL ready for judging
- [ ] If the repository remains private, grant `testing@devpost.com` and
  `build-week-event@openai.com` access before the submission deadline
- [ ] Start and maintain the editable Devpost submission before the deadline
- [ ] Confirm every teammate has accepted their Devpost project invitation
- [ ] README includes Codex collaboration details
- [ ] Demo video stays under 3 minutes
- [ ] Demo video clearly shows the product and the Codex/GPT-5.6 contribution
- [ ] Verify the final video link in a private/incognito browser session; unlisted is acceptable
- [ ] `/feedback` session ID captured from the main Codex build thread
- [ ] Build-week commit history clearly shows July 13-21, 2026 work

## Active Next Steps

- Improve freeform handwriting extraction from arbitrary photos
- Replace placeholder font generation artifacts with real vector/font assembly
- Integrate the Rust wrapper with a compiled WASM delivery path
- Add real image decoding and font export behavior beyond the current placeholder artifact

## Known Risks

- Real handwriting-to-font generation is substantially more complex than the current scaffold
- A freeform image has no reliable glyph-to-character mapping without OCR, transcription, or a
  controlled sample; v1 therefore uses image-derived style with a legible glyph grammar rather
  than claiming literal per-character reconstruction
- Browser-side processing must be kept performant enough for Vercel Hobby delivery
- CJK and other large scripts remain a roadmap item, not current functionality
- Frontend `npm audit` currently reports a moderate transitive `postcss < 8.5.10` issue through `next@16.2.10`; do not use `npm audit fix --force` because the suggested downgrade path is invalid for this project
