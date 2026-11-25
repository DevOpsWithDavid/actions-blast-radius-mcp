# Decisions

## Decided (25 Nov 2026)
- Language: Rust for scanner and MCP server; SQLite file for the graph. Eval runner may be Python or TypeScript.
- Time budget: 6 to 8 hrs/week, 6 weeks.
- Hosting: GitHub org DevOpsWithDavid (demo corpus plus this repo).
- Name: `actions-blast-radius-mcp`.
- Graph: ontology-lite (no RDF/OWL/SPARQL, no distributed store).
- License: MIT (simple, standard default for a public portfolio project; no reason found to prefer Apache-2.0's patent grant here).

## Decided (25 Nov 2026, week-1 spike)
- **MCP SDK: `rmcp`** (`modelcontextprotocol/rust-sdk`). Official, Tier 1 on modelcontextprotocol.io's SDK list, tokio-based, actively released (recent changelog entries into mid-2026). No serious alternative found; this is the only Rust SDK with official status.
- **GitHub API client: `octocrab`.** De facto standard Rust GitHub client (120+ published versions since 2020, 5.6M+ downloads, commits as recent as Sep 2026). Handles pagination and auth; we will layer our own rate-limit backoff per `docs/spec.md`'s risk list.
- **YAML parsing: `serde-saphyr`**, not `serde_yaml`. `serde_yaml` is deprecated by its maintainer and sits on `unsafe-libyaml` (RUSTSEC-2024-0449). `serde-saphyr` is the actively maintained, memory-safe, serde-integrated successor with better error diagnostics; used for both workflow YAML (`workflow_parser`) and scenario YAML (`docs/scenario-spec.md`). (Rejected: `yaml-rust2`/`saphyr` — DOM-level parsers, no serde derive support, more boilerplate for typed structs; `serde_yaml_ng`/`yaml_serde` — stopgap forks still built on the same unsafe C core.)
- **SQLite: `rusqlite`**, synchronous. Standard choice for an embedded, single-process CLI/server (no DB server, per `AGENTS.md`). The MCP server runs queries from async tool handlers via `tokio::task::spawn_blocking` rather than pulling in `sqlx`'s async-but-2x-slower-per-query overhead for no benefit here.
- **Migrations: `rusqlite_migration`.** Single-maintainer but healthy cadence (dependency bumps merged within the same week as this decision), uses SQLite's `user_version` pragma instead of a tracking table — simpler than `refinery` for a single embedded `.db` file.
- **Workspace layout:** `scanner/`, `server/`, `graph/` (shared types + SQLite access), matching `AGENTS.md`'s rule 4 (eval generator must not import graph-builder code — the generator lives under `eval/` and does not depend on `graph`).
- **Spike verification:** `cargo check --workspace` built clean on first try (after bumping `serde-saphyr` off its unresolvable `0.1` guess to `2.2`). Resolved versions: `rmcp 0.9.1`, `octocrab 0.54.9`, `serde-saphyr 2.2.0`, `rusqlite 0.43.0`, `rusqlite_migration 2.9.0`, `tokio 1.54.1`. Pinned in the workspace `Cargo.toml`.

## Proposed (to confirm)
- Jev as an optional pluggable judge, week 5 stretch.

## Open
- Exact repo names in the org; check for clashes.
- Questions from the coding agent go here with date and context.
