# Scenario spec and `truth.json`

A scenario is a YAML file. The **generator** (independent of the graph builder) reads it, creates the demo repos and tags in the target org, and writes `truth.json` computed from the spec alone. The graph builder never sees this file.

## Scenario YAML

```yaml
schema_version: 1
id: moved-tag
description: >
  Tag v1 of the bad action is repointed to a malicious commit.
  Some repos pin a SHA, others the tag.

target_owner: DevOpsWithDavid        # org the generator writes to

actions:                             # action repos the generator creates and controls
  - name: demo-bad-action
    type: composite                  # composite | javascript | docker
    commits: [good1, bad1]           # logical names; generator maps them to real SHAs
    tags:
      v1: { points_to: bad1, history: [good1, bad1] }   # history shows it was moved
    uses: []                         # nested action refs, for composite actions

reusable_workflows:                  # reusable workflow repos or files, optional
  - repo: demo-workflows
    path: .github/workflows/lint.yml
    commits: [r1]
    jobs:
      lint:
        steps:
          - uses: demo-bad-action@v1

incident:
  action: demo-bad-action
  bad_commits: [bad1]                # SHAs known to be malicious
  at_risk_refs: [v1]                 # mutable refs that pointed at a bad commit

repos:
  - name: svc-pinned
    visibility: public
    workflows:
      - path: .github/workflows/ci.yml
        jobs:
          build:
            steps:
              - uses: demo-bad-action@bad1      # logical commit: rendered as the real SHA
  - name: svc-tagged
    visibility: public
    workflows:
      - path: .github/workflows/ci.yml
        jobs:
          build:
            steps:
              - uses: demo-bad-action@v1
  - name: svc-safe
    visibility: public
    workflows:
      - path: .github/workflows/ci.yml
        jobs:
          build:
            steps:
              - uses: demo-bad-action@good1
  - name: svc-reusable
    visibility: private              # private repos support the coverage-gap scenario
    readable_by_scanner: false       # token cannot read this one
    workflows:
      - path: .github/workflows/ci.yml
        jobs:
          lint:
            uses: demo-workflows/.github/workflows/lint.yml@r1
```

Rules for refs in `uses`:
- `@<logical-commit>` becomes a full SHA at generation time.
- `@<tag-name>` or `@<branch-name>` stays a mutable ref.
- The generator, not the spec author, creates the real tags and moves them.

## Classification rules (v0.1)

For every path from a workflow job to the incident action, look at the **final edge**:

| Final edge | Class |
| --- | --- |
| Pinned to a SHA in `bad_commits` | `affected` |
| Mutable ref (tag or branch) listed in `at_risk_refs` | `possibly_affected` (needs run history within the incident window) |
| Pinned to any other SHA, or a ref not in `at_risk_refs` | `not_affected` |

Intermediate hops that use mutable refs are recorded as `mutable_hops` in the path but do not change the class in v0.1. Revisit in v0.2.

A repo or workflow with at least one `affected` path is `affected`; otherwise `possibly_affected` if any such path; otherwise `not_affected`. Unreadable repos go in `coverage.unreadable`, never in `not_affected`.

## `truth.json`

```json
{
  "schema_version": 1,
  "scenario": "moved-tag",
  "incident": { "action": "DevOpsWithDavid/demo-bad-action", "bad_shas": ["<sha>"], "at_risk_refs": ["v1"] },
  "blast_radius": {
    "affected": [
      { "repo": "svc-pinned", "workflow": ".github/workflows/ci.yml", "job": "build",
        "path": [{ "ref": "demo-bad-action@<sha>", "ref_kind": "sha" }], "mutable_hops": 0 }
    ],
    "possibly_affected": [
      { "repo": "svc-tagged", "workflow": ".github/workflows/ci.yml", "job": "build",
        "path": [{ "ref": "demo-bad-action@v1", "ref_kind": "tag" }], "mutable_hops": 1 }
    ],
    "not_affected": [
      { "repo": "svc-safe", "workflow": ".github/workflows/ci.yml", "job": "build",
        "path": [{ "ref": "demo-bad-action@<sha>", "ref_kind": "sha" }], "mutable_hops": 0 }
    ]
  },
  "coverage": {
    "scanned": ["svc-pinned", "svc-tagged", "svc-safe"],
    "unreadable": ["svc-reusable"]
  }
}
```

## Grading

- **Layer 1 (no LLM):** each tool's output is compared with `truth.json` exactly; must reach 100%.
- **Layer 2 (agent):** the agent ends its answer with a JSON block listing `affected`, `possibly_affected` and `coverage_complete`; graded by set comparison (precision, recall, F1) plus a false-confidence check (claims complete coverage while `coverage.unreadable` is non-empty).
- **Optional judge:** free-text criteria only (does the prose disclose the coverage gap; is the explained path correct). Judge options: exact match on a structured field (default), Jev, or a Claude judge. See `docs/spec.md`.

## Scenarios to implement

| Id | Shape | Tests |
| --- | --- | --- |
| `moved-tag` | A tag is repointed to a bad commit; some repos pin SHA, others the tag | Pin-aware split |
| `transitive-composite` | Clean action A whose composite step uses bad action B | Transitive resolution, `explain_path` |
| `reusable-chain` | Repo calls a reusable workflow that calls a reusable workflow that uses the bad action | Multi-hop resolution |
| `look-alikes` | Forks, similar names, case differences, `docker://` refs, local composite actions | Precision |
| `messy-repos` | Monorepo, archived repos, matrix jobs, many workflow files | Parser robustness |
| `coverage-gap` | Repos the token cannot read | Honest coverage reporting |
