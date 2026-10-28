# ADR 0002: Dependencies for the notebook model

- Status: Proposed
- Date: 2026-10-23

## Context

PRD-001 builds the notebook model in `gyeoldaero-core` and `gyeoldaero check`. It needs a
few crates the dependency table in `architecture.md` doesn't list: config is TOML, slugs
and link targets compare after NFC normalization (PRD-001 Requirement 8), and the CLI
tests need to run the binary. It also uses `comrak` in core, which the table assigns to
rendering.

## Decision

- `toml` to parse `gyeoldaero.toml`. It is read as a `toml::Table` rather than through
  `serde` derive, so unknown settings can be reported as warnings; `serde` is not a direct
  dependency.
- `unicode-normalization` for NFC comparison keys.
- `comrak` (default features off) in `gyeoldaero-core`, used only to parse Markdown into
  an AST for wiki link extraction. Core still produces no HTML.
- `assert_cmd` as a dev-dependency of `gyeoldaero-cli` for exit-code and output tests.
- `serde_yaml_ng` 0.10 for frontmatter, as Ellie decided in PRD-001.
- `dirs` for the platform cache dir (default `build.out`) and the home dir (`~`
  expansion), as Ellie decided in PRD-001.

## Consequences

- Core and render agree on what counts as a link: both read the same comrak AST, so a
  `[[…]]` inside code is ignored by both.
- Core compiles comrak; build time goes up slightly. With default features off it pulls
  no CLI or syntax-highlighting dependencies.
- `comrak` versions must stay in step between core and render (one workspace
  dependency).

## Alternatives considered

- Hand-written link scanner in core: would have to mirror CommonMark's code-span and
  fence rules and could disagree with the renderer.
- `icu_normalizer` instead of `unicode-normalization`: larger, more than we need.
- Reading `XDG_CACHE_HOME` / `HOME` by hand instead of `dirs`: wrong on macOS
  (`~/Library/Caches`) and Windows.
- `trycmd` / `snapbox` for CLI tests: more than we need for exit codes and a few lines of
  output.
