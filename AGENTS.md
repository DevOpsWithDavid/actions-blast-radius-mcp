# AGENTS.md: brief for coding agents

You are helping David build `actions-blast-radius-mcp`, a Rust MCP server plus eval harness. Read this file first, then `docs/plan.md` to see what to work on.

## What this project is

A scanner and MCP server that answers "what is affected by this compromised GitHub Action?" across an org, with transitive resolution and pin-aware classification, proven by an eval with exact ground truth. It is a public portfolio project aimed at Anthropic's AI Reliability Engineering and Forward Deployed Engineer roles, so quality, tests and honest results matter more than feature count.

Where things are:
- `docs/spec.md`: scope, architecture, tool contracts, eval design
- `docs/ontology.md`: the graph model; the schema is the contract
- `docs/scenario-spec.md`: eval scenario YAML and `truth.json`
- `docs/plan.md`: milestones as checkboxes
- `docs/decisions.md`: decisions already made; do not relitigate without asking

## Hard rules

1. **Clean-room.** No code, data, screenshots or repo names from any employer or client. Build only from public sources and from what David writes here. If something looks like it came from an employer, stop and ask.
2. **Read-only against GitHub.** The scanner and server only read. Never write a workflow that pushes, and never request broader token scopes than needed. Tokens come from environment variables and are never logged, committed or put in fixtures.
3. **Scope discipline.** v0.1 is GitHub Actions only. Lockfiles, advisory feeds, auto-remediation, GitLab and any UI are out of scope; park ideas in `docs/v0.2.md`.
4. **Independent answer key.** The eval generator must not share code with the graph builder, or the eval tests nothing. Keep them in separate crates or modules with no imports between them.
5. **Honest coverage.** Every tool response includes the `graph_status` summary. Never let a tool claim completeness when repos were skipped.
6. **No unverified dependencies.** Crate and SDK choices (MCP server SDK, GitHub API client, YAML, SQLite) are unconfirmed. Do the week-1 spike in `docs/plan.md`, check maintenance status, and record the choice in `docs/decisions.md` before building on it.

## Stack

- Rust for the scanner and MCP server; SQLite file for the graph (no database server).
- The eval runner may be Python or TypeScript because agent SDK examples are in those languages.
- David is still learning Rust: prefer clear, idiomatic code with short comments on non-obvious ownership or lifetime choices over clever abstractions.

## How to work

- Pick the first unticked item in `docs/plan.md`, do it, and tick it. Keep changes small and each one testable.
- Write tests with the code. Layer 1 (tools without an LLM, compared against `truth.json`) must reach 100%.
- Commit messages: imperative, one line, plus a short body when the why is not obvious.
- When a spec detail is ambiguous, add the question to `docs/decisions.md` under "Open" and ask David rather than guessing.
- Update docs in the same change as the code they describe.

## Definition of done for v0.1

- Public repo with install steps and a demo recording
- Layer 1 tests at 100% against `truth.json`
- Agent eval results table in the README (baseline vs with server), whatever the numbers
- One English write-up of the method and what the baseline got wrong
