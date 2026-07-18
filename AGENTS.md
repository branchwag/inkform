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
- For strongly cursive samples, generated alphabetic glyphs include a thin exit stroke that
  overlaps the following glyph and use tighter side bearings. Keep it as one shared contextual
  connection strategy, not a full baseline underline or disconnected decorative ligatures
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
- If the browser starts showing a generic placeholder preview again, first verify that the WASM bundle was rebuilt from current Rust sources
- In restricted sandboxes, `wasm-pack` may fail because `wasm-bindgen` install/cache paths are not writable or network access is unavailable; a local machine run is the source of truth for browser-side validation
- The current preview fallback path is `svg-v3`, which is tuned to visually match the generated TTF more closely when the browser cannot load the font directly
- FreeType-based validation (`fc-scan`, ImageMagick render, `woff2_compress`) currently accepts the generated TTF, but Zen browser may still reject direct `FontFace` loading and fall back to SVG preview

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
- [ ] README includes Codex collaboration details
- [ ] Demo video stays under 3 minutes
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
