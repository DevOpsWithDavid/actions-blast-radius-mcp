# Graph model (ontology-lite)

The schema is the contract. Rust types and the SQLite tables must match this file; change this file first.

Inspired by the discipline of Netflix's observability knowledge graph (explicit ontology, enricher passes, validation, snapshots), not its technology: there is no RDF, OWL or SPARQL here. See [talk coverage](https://www.infoq.com/news/2026/03/ontology-at-netflix).

## Entities (nodes)

| Node | Key | Properties |
| --- | --- | --- |
| `Repo` | `owner/name` | visibility, archived, default_branch, readable (bool) |
| `Workflow` | `repo + path` | path, is_reusable (has `workflow_call`) |
| `Job` | `workflow + job_id` | job_id, calls_workflow (optional `uses:` at job level) |
| `ActionRef` | `owner/name[/path]@ref` as written | owner, name, path, ref, ref_kind (`sha`, `tag`, `branch`, `local`, `docker`) |
| `Commit` | `owner/name@sha` | sha |

`ref_kind` rules: 40 hex characters is `sha`; a ref that exists as a tag is `tag`; otherwise `branch`. `./path` is `local`; `docker://` is `docker`. When a ref cannot be classified (for example, the API call failed), record `unknown` and count it in `graph_status`; never guess.

## Relationships (edges)

| Edge | From to | Meaning |
| --- | --- | --- |
| `CONTAINS` | Repo to Workflow, Workflow to Job | Structure |
| `USES` | Job to ActionRef | A step in the job uses this action |
| `CALLS` | Job to ActionRef | The job calls a reusable workflow |
| `INCLUDES` | ActionRef (composite) to ActionRef | A composite action's own steps, read at the resolved commit |
| `RESOLVES_TO` | ActionRef to Commit | What the ref pointed to at scan time |

Transitive resolution follows `USES` / `CALLS`, then `RESOLVES_TO`, then reads the action's definition (`action.yml`) or the reusable workflow file at that commit and follows its own `USES` / `INCLUDES`. Cycle protection is required; record the cycle and stop.

## Enrichers (ingest passes)

Each pass writes facts with `source` and `scan_id`:

1. `workflow_parser`: workflow YAML to Workflow, Job, ActionRef (`USES`, `CALLS`)
2. `ref_resolver`: ActionRef to Commit (`RESOLVES_TO`), and `ref_kind` classification
3. `definition_fetcher`: action definition or reusable workflow at the resolved commit to `INCLUDES` / nested `USES`

## Validation (checked at ingest)

Facts that violate these are rejected and counted in `graph_status.rejected_facts`:

- An ActionRef has exactly one `ref_kind`.
- A `RESOLVES_TO` edge exists only for `tag`, `branch` and `sha` refs.
- A Job belongs to exactly one Workflow; a Workflow to exactly one Repo.
- `INCLUDES` only originates from a composite action.

## Snapshots

Every fact carries a `scan_id`; scans are never overwritten. This enables the v0.2 question "which tags moved since the last scan?" by comparing `RESOLVES_TO` between scans. In v0.1 the data model supports it but no tool exposes it.

## SQLite sketch (illustrative; refine during implementation)

```sql
CREATE TABLE scan (
  scan_id     INTEGER PRIMARY KEY,
  started_at  TEXT NOT NULL,
  owner       TEXT NOT NULL
);
CREATE TABLE repo (
  scan_id INTEGER NOT NULL, repo TEXT NOT NULL,
  visibility TEXT, archived INTEGER, readable INTEGER NOT NULL,
  PRIMARY KEY (scan_id, repo)
);
CREATE TABLE workflow (
  scan_id INTEGER NOT NULL, repo TEXT NOT NULL, path TEXT NOT NULL, is_reusable INTEGER NOT NULL,
  PRIMARY KEY (scan_id, repo, path)
);
CREATE TABLE job (
  scan_id INTEGER NOT NULL, repo TEXT NOT NULL, path TEXT NOT NULL, job_id TEXT NOT NULL,
  PRIMARY KEY (scan_id, repo, path, job_id)
);
CREATE TABLE action_ref (
  scan_id INTEGER NOT NULL, ref_id INTEGER NOT NULL,
  owner TEXT, name TEXT, subpath TEXT, ref TEXT, ref_kind TEXT NOT NULL,
  PRIMARY KEY (scan_id, ref_id)
);
CREATE TABLE edge (
  scan_id INTEGER NOT NULL, kind TEXT NOT NULL,          -- USES, CALLS, INCLUDES, RESOLVES_TO
  from_kind TEXT NOT NULL, from_key TEXT NOT NULL,
  to_kind TEXT NOT NULL,   to_key TEXT NOT NULL,
  source TEXT NOT NULL                                   -- which enricher wrote it
);
```

Transitive queries use recursive CTEs over `edge`.
