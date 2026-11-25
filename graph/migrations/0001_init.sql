-- Schema from docs/ontology.md. That file is the contract: change it
-- first, then this migration, then the Rust types in graph/src/model.rs.
--
-- Every table carries scan_id and scans are append-only (see "Snapshots"
-- in docs/ontology.md) -- a scan is never updated or deleted in place.

CREATE TABLE scan (
    scan_id    INTEGER PRIMARY KEY,
    started_at TEXT NOT NULL,
    owner      TEXT NOT NULL
);

CREATE TABLE repo (
    scan_id    INTEGER NOT NULL REFERENCES scan (scan_id),
    repo       TEXT NOT NULL,
    visibility TEXT,
    archived   INTEGER NOT NULL DEFAULT 0,
    readable   INTEGER NOT NULL,
    PRIMARY KEY (scan_id, repo)
);

CREATE TABLE workflow (
    scan_id     INTEGER NOT NULL REFERENCES scan (scan_id),
    repo        TEXT NOT NULL,
    path        TEXT NOT NULL,
    is_reusable INTEGER NOT NULL,
    PRIMARY KEY (scan_id, repo, path),
    FOREIGN KEY (scan_id, repo) REFERENCES repo (scan_id, repo)
);

CREATE TABLE job (
    scan_id INTEGER NOT NULL REFERENCES scan (scan_id),
    repo    TEXT NOT NULL,
    path    TEXT NOT NULL,
    job_id  TEXT NOT NULL,
    PRIMARY KEY (scan_id, repo, path, job_id),
    FOREIGN KEY (scan_id, repo, path) REFERENCES workflow (scan_id, repo, path)
);

-- One row per distinct "owner/name[/subpath]@ref" string seen in a scan.
-- ref_kind values match docs/ontology.md's classification rules exactly;
-- the rules themselves live in the scanner's ref_resolver (week 2), not here.
-- owner/name/ref are nullable, matching the ontology's SQL sketch exactly:
-- a `local` ref (./path) or `docker://` ref has no GitHub owner/name/ref at
-- all. See the open question in docs/decisions.md about what local/docker
-- refs put in subpath.
CREATE TABLE action_ref (
    scan_id  INTEGER NOT NULL REFERENCES scan (scan_id),
    ref_id   INTEGER NOT NULL,
    owner    TEXT,
    name     TEXT,
    subpath  TEXT,
    ref      TEXT,
    ref_kind TEXT NOT NULL CHECK (ref_kind IN ('sha', 'tag', 'branch', 'local', 'docker', 'unknown')),
    PRIMARY KEY (scan_id, ref_id)
);
CREATE INDEX idx_action_ref_lookup ON action_ref (scan_id, owner, name);

-- Generic edge table: CONTAINS / USES / CALLS / INCLUDES / RESOLVES_TO.
-- Endpoints are (kind, key) pairs rather than per-table foreign keys,
-- because any edge can target any node kind in the ontology -- see
-- "Relationships" in docs/ontology.md. `source` names the enricher pass
-- that wrote the fact (docs/ontology.md "Enrichers").
CREATE TABLE edge (
    scan_id   INTEGER NOT NULL REFERENCES scan (scan_id),
    kind      TEXT NOT NULL CHECK (kind IN ('CONTAINS', 'USES', 'CALLS', 'INCLUDES', 'RESOLVES_TO')),
    from_kind TEXT NOT NULL,
    from_key  TEXT NOT NULL,
    to_kind   TEXT NOT NULL,
    to_key    TEXT NOT NULL,
    source    TEXT NOT NULL
);
CREATE INDEX idx_edge_from ON edge (scan_id, from_kind, from_key);
CREATE INDEX idx_edge_to ON edge (scan_id, to_kind, to_key);

-- Facts rejected at ingest (docs/ontology.md "Validation"), kept so
-- graph_status.rejected_facts (week 3) can report a count without
-- re-walking the scan.
CREATE TABLE rejected_fact (
    scan_id INTEGER NOT NULL REFERENCES scan (scan_id),
    reason  TEXT NOT NULL,
    detail  TEXT NOT NULL
);
