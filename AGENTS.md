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
- Browser-side processing must be kept performant enough for Vercel Hobby delivery
- CJK and other large scripts remain a roadmap item, not current functionality
- Frontend `npm audit` currently reports a moderate transitive `postcss < 8.5.10` issue through `next@16.2.10`; do not use `npm audit fix --force` because the suggested downgrade path is invalid for this project
