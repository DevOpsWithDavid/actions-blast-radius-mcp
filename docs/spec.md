# v0.1 spec (condensed)

Discussion copy with diagrams: https://claude.ai/artifact/3GVkcEAurttuNW45WBUZZW. This file is the working copy for the coding agent; if they disagree, ask in `docs/decisions.md`.

## Pitch

An MCP server that answers "what in my GitHub org is affected by this compromised GitHub Action?" from a transitive dependency graph, with pin-aware classification, and an eval with exact ground truth that measures how much better an agent does with the server than without it.

## Scope (v0.1)

In: scan an org's workflows, build the graph (SQLite), serve five read-only MCP tools, run the eval (Layer 1 deterministic, Layer 2 agent arms).
Out (park in `docs/v0.2.md`): run-history checks, snapshot diff tools ("which tags moved"), write actions, non-GitHub CI, a UI.

## Architecture

```
GitHub API (read-only token from env)
        |
   scanner CLI (Rust)  -- enrichers: workflow_parser, ref_resolver, definition_fetcher
        |
   graph.db (SQLite, scan_id on every fact)
        |
   MCP server (Rust) -- find_usage, blast_radius, explain_path, pin_report, graph_status
        |
   agent (Claude etc.)

eval/: generator (scenario YAML -> demo repos + truth.json)  ->  runner (Layer 1, Layer 2)
```

The generator and the graph builder share no code. See `docs/ontology.md` and `docs/scenario-spec.md`.

## MCP tools (all read-only; every response includes `graph_status`)

| Tool | Input | Output |
| --- | --- | --- |
| `find_usage` | action | Every job that uses it, with ref and pin type |
| `blast_radius` | action, `bad_shas[]`, `at_risk_refs[]` | `affected`, `possibly_affected`, `not_affected`, each with reasons and paths |
| `explain_path` | repo, workflow, job, action | The chain from the job to the action, including mutable hops |
| `pin_report` | optional repo or org filter | Counts and lists by pin type (SHA, tag, branch) |
| `graph_status` | none | Scan time, repos scanned, repos skipped or unreadable, rejected facts |

Classification rules are in `docs/scenario-spec.md`.

## Competitive check (not exhaustive)

Existing: github-show-actions, StepSecurity inventory, GitHub's dependency graph for Actions, gh-actions-dependency-graph, ActionScope, agent-bom, code-impact-mcp / whobreaks, dtrack-mcp. None found that combines an org-wide transitive Actions graph, pin-aware classification, an MCP interface, and measured accuracy. Clean-room rule: read their READMEs for positioning only; do not copy code.

## Eval design

- **Layer 1 (no LLM):** run each tool against the generated corpus and compare with `truth.json`. Must be 100%.
- **Layer 2 (agent arms):** baseline (agent with `gh` and code search) vs agent with the MCP server. 5 runs per cell. Metrics: precision, recall, F1, false-confidence rate, tool calls, tokens, time. Phase 2 repeats under fault injection (rate limits, truncated results).
- **Optional judge (week 5 stretch, proposed, to confirm):** pluggable judge for free-text criteria: structured exact match (default), Jev, Claude. Accept Jev's verdict only when confident, otherwise escalate to Claude.

## Risks and open questions

- Static graph cannot know when workflows ran: hence `possibly_affected`.
- Private repos and token scope: hence `graph_status` coverage in every response.
- Rate limits when resolving many refs: cache by `owner/name@ref` within a scan.
- Rust MCP SDK maturity: week-1 spike, record choice in `docs/decisions.md`.
- Demo corpus lives in the DevOpsWithDavid org: one skeletal microservices monorepo plus about 10 small repos (an action we control so a tag can be moved safely, a reusable-workflow library, a composite action, one private repo for the coverage-gap scenario).
