# Inkform

Inkform is a Rust-first web app that turns guided handwriting samples into a downloadable font. The current build targets a Vercel Hobby deployment with a Next.js frontend and a Rust workspace that is structured for WebAssembly execution in the browser.

## Status

This repository currently contains the production scaffold for the hackathon build:

- Rust workspace with a core engine crate and a WASM-facing wrapper crate
- Test suite for the Rust core and wrapper layers
- Clippy, formatting, and test quality gates
- Next.js frontend scaffold with a product-facing landing flow
- `AGENTS.md` as the durable project context log

## Repository Layout

- `crates/inkform-core`: deterministic Rust domain logic for validation, glyph extraction planning, normalization, preview, and generation orchestration
- `crates/inkform-wasm`: browser-facing Rust wrapper around the core crate
- `frontend`: Next.js app intended for Vercel Hobby hosting
- `.github/workflows/ci.yml`: CI checks for formatting, clippy, and tests
- `AGENTS.md`: persistent context and decision ledger for Codex sessions

## Quality Gates

Run these before every commit:

```bash
./scripts/check.sh
```

The frontend should also be checked once dependencies are installed:

```bash
cd frontend
npm install
npm run lint
npm run typecheck
```

To build the browser WASM package:

```bash
bash scripts/build-wasm.sh
```

## Hackathon Notes

- Primary track: `Apps for Your Life`
- Current guaranteed language coverage: Latin extended
- Future non-Latin expansion will use script packs rather than claiming full Unicode support up front
- Keep the majority of build work in one Codex thread so the `/feedback` session ID remains submission-ready

## Codex Collaboration

This section should be expanded as the build progresses. It exists now so the final submission has a clear place to document how Codex and GPT-5.6 were used for:

- architecture and scope decisions
- Rust workspace and testing setup
- frontend scaffolding and integration
- quality review, refactoring, and release prep

## Git Note

The current sandbox exposes a read-only `.git` path, so writable git metadata could not be initialized from inside this session. The project files are fully scaffolded, but git initialization may need to be completed outside the sandbox or with a separate writable git directory.
