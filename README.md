# actions-blast-radius-mcp

An MCP server that answers "what in my GitHub org is affected by this compromised action?" in one query. It scans an org's GitHub Actions workflows into a dependency graph (including reusable workflows and composite actions), classifies every hit by how it is pinned, and ships with an eval that has exact ground truth.

**Status:** pre-v0.1, spec stage. Nothing is built yet. Start with `docs/plan.md`.

## The question it answers

> A popular action was compromised and its `v1` tag was repointed to a malicious commit. Which of our repos are affected, which are possibly affected, and why?

A flat "who uses this action" list gets this wrong in two ways: it misses transitive use (an action inside a composite action inside a reusable workflow), and it treats a SHA-pinned repo and a tag-pinned repo as the same risk.

## MCP tools (v0.1, all read-only)

| Tool | Question it answers |
| --- | --- |
| `find_usage` | Where is this action used, directly or transitively? |
| `blast_radius` | What is affected, possibly affected, or not affected by this compromised action? |
| `explain_path` | Why is this repo affected? (repo, workflow, job, reusable workflow or composite action, target) |
| `pin_report` | How well pinned is this repo or org (SHA, tag, branch)? |
| `graph_status` | Can I trust this answer? (scan time, repos covered, repos skipped and why) |

Graph building is a separate CLI (`scan`), so the server never needs write access or broad token scopes.

## Why another tool

Existing tools cover pieces: org-wide action inventories, per-workflow graphs, AWS reach analysis, file-level impact analysis for coding agents. None found that combine transitive resolution, pin-aware classification, an MCP interface and a measured accuracy. See `docs/spec.md` for the competitor table.

## Clean-room rule

This project is built from scratch on public data. No code, data, screenshots or repo names from any employer or client may be used or referenced, ever.

## Docs

- `docs/spec.md`: scope, architecture, tools, eval design, risks
- `docs/ontology.md`: the graph model (entities, edges, validation, snapshots)
- `docs/scenario-spec.md`: the YAML format for eval scenarios and the `truth.json` format
- `docs/plan.md`: six-week plan with checkboxes
- `docs/decisions.md`: decisions made so far
- `AGENTS.md`: brief for coding agents working in this repo

## License

MIT, see `LICENSE`.
