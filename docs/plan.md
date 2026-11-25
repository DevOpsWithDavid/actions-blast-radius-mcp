# Plan (6 weeks, 6 to 8 hrs/week)

Work the first unticked item. Tick it when done and tests pass. Park new ideas in `docs/v0.2.md`.

## Week 1: foundations and spike
- [x] Spike: confirm a Rust MCP SDK, GitHub API client, YAML parser and SQLite crates; check maintenance status and record choices in `docs/decisions.md`
- [x] Cargo workspace: `scanner`, `server`, shared `graph` crate
- [x] SQLite schema from `docs/ontology.md` with migrations
- [ ] Workflow parser for `uses:` in steps and jobs, with unit tests on fixture YAML

## Week 2: scanner
- [ ] Org enumeration with pagination and rate-limit handling; token from env only
- [ ] `ref_resolver`: tag, branch, SHA classification and resolution to commits
- [ ] `definition_fetcher`: composite actions and reusable workflows at resolved commits, with cycle protection
- [ ] Validation at ingest; `graph_status` counters

## Week 3: MCP server
- [ ] Server skeleton and `graph_status`
- [ ] `find_usage`, `pin_report`
- [ ] `blast_radius` with the classification rules
- [ ] `explain_path`

## Week 4: eval generator and Layer 1
- [ ] Scenario loader and validator (`docs/scenario-spec.md`)
- [ ] Generator: creates repos, commits and tags in the org; writes `truth.json` without using graph-builder code
- [ ] Scenarios: moved-tag, transitive-composite, reusable-chain
- [ ] Layer 1 runner at 100%

## Week 5: Layer 2 and scenarios
- [ ] Scenarios: look-alikes, messy-repos, coverage-gap
- [ ] Agent runner for baseline and MCP arms, 5 runs per cell, metrics output
- [ ] Stretch: pluggable judge (exact match default; Jev; Claude)

## Week 6: polish and release
- [ ] Fault-injection phase
- [ ] README results table (baseline vs server, whatever the numbers are)
- [ ] Install instructions, demo recording, license, tag v0.1
- [ ] Write-up draft
