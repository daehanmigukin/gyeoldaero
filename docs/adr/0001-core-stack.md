# ADR 0001: Core stack

- Status: Accepted
- Date: 2026-10-23

## Context

gyeoldaero is a single self-hosted binary that scans a folder of Markdown, renders static
HTML, watches for changes and serves the result. Ellie chose Rust. The renderer must
support user theming later and must render untrusted Markdown safely.

## Decision

- Rust, edition 2024, stable toolchain, Cargo workspace of four crates
  (`gyeoldaero-core`, `gyeoldaero-render`, `gyeoldaero-serve`, `gyeoldaero-cli`) with one-way dependencies.
- `comrak` for Markdown (GFM, wikilinks extension, raw HTML disabled).
- `minijinja` for templates, loaded at runtime from embedded defaults, so themes can
  override templates later.
- `notify` + `notify-debouncer-full` for watching.
- `axum` + `tower-http` + `tokio` for serving; core and render stay synchronous.
- `thiserror` in libraries, `anyhow` in the binary, `tracing` for logs, `insta` for
  snapshot tests.
- Static multi-page output with a progressive-enhancement script, not a single-page app.

## Consequences

- Pages work without JavaScript and are cacheable static files.
- The interaction layer must fetch and swap pages to animate, which adds router logic to
  `gyeoldaero.js` (see `renderer-spec.md` §4.1).
- Runtime templates cost a little startup time compared with compile-time templates
  (askama); accepted for theming flexibility.

## Alternatives considered

- Emanote / existing static site generators: tried; navigation model didn't fit.
- Gemini/gemtext as primary output: dropped; no collapsible content or gestures.
- Go: matches Haven, but Rust fits the yazi ecosystem and Ellie's preference here.
