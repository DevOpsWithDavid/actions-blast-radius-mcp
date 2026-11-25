//! Shared graph types and SQLite access for actions-blast-radius-mcp.
//!
//! `docs/ontology.md` is the contract: if this crate and that file
//! disagree, fix the doc first, then `migrations/0001_init.sql`, then
//! this code.

pub mod model;

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("../migrations/0001_init.sql"))])
}

/// A handle to one SQLite file holding one or more scans' worth of graph
/// data. Every fact carries a `scan_id`; scans are append-only (see
/// "Snapshots" in `docs/ontology.md`), so a `Graph` can accumulate many
/// scans over time without losing history.
pub struct Graph {
    conn: Connection,
}

impl Graph {
    /// Open (creating if needed) the SQLite file at `path` and bring its
    /// schema up to date.
    pub fn open(path: impl AsRef<Path>) -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open(path)?;
        migrations().to_latest(&mut conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database. Used by tests and by any caller that
    /// only needs a throwaway graph for a single run.
    pub fn open_in_memory() -> rusqlite_migration::Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        migrations().to_latest(&mut conn)?;
        Ok(Self { conn })
    }

    /// Record the start of a new scan and return its `scan_id`. Scans are
    /// never updated or deleted once started; `started_at` should be an
    /// RFC 3339 timestamp.
    pub fn start_scan(&self, owner: &str, started_at: &str) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO scan (started_at, owner) VALUES (?1, ?2)",
            (started_at, owner),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Direct access to the underlying connection, for enrichers and MCP
    /// tools that need queries this crate doesn't wrap yet.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_exists(graph: &Graph, name: &str) -> bool {
        graph
            .conn()
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0
    }

    #[test]
    fn migrations_create_every_table_from_the_ontology() {
        let graph = Graph::open_in_memory().unwrap();
        for table in ["scan", "repo", "workflow", "job", "action_ref", "edge", "rejected_fact"] {
            assert!(table_exists(&graph, table), "missing table: {table}");
        }
    }

    #[test]
    fn start_scan_returns_increasing_ids_and_never_rewrites() {
        let graph = Graph::open_in_memory().unwrap();
        let first = graph.start_scan("DevOpsWithDavid", "2026-11-25T00:00:00Z").unwrap();
        let second = graph.start_scan("DevOpsWithDavid", "2026-11-25T00:05:00Z").unwrap();
        assert!(second > first);

        let scans: i64 = graph
            .conn()
            .query_row("SELECT count(*) FROM scan", [], |row| row.get(0))
            .unwrap();
        assert_eq!(scans, 2, "both scans should be kept, not overwritten");
    }

    #[test]
    fn action_ref_allows_null_owner_name_ref_for_local_and_docker_kinds() {
        // ./path and docker://... refs have no GitHub owner/name/ref; see
        // the open question in docs/decisions.md about what subpath holds.
        let graph = Graph::open_in_memory().unwrap();
        let scan_id = graph.start_scan("DevOpsWithDavid", "2026-11-25T00:00:00Z").unwrap();
        let result = graph.conn().execute(
            "INSERT INTO action_ref (scan_id, ref_id, owner, name, subpath, ref, ref_kind)
             VALUES (?1, 1, NULL, NULL, './.github/actions/build', NULL, 'local')",
            [scan_id],
        );
        assert!(result.is_ok(), "local refs must not require owner/name/ref: {result:?}");
    }

    #[test]
    fn action_ref_rejects_an_unknown_ref_kind_value() {
        let graph = Graph::open_in_memory().unwrap();
        let scan_id = graph.start_scan("DevOpsWithDavid", "2026-11-25T00:00:00Z").unwrap();
        let result = graph.conn().execute(
            "INSERT INTO action_ref (scan_id, ref_id, owner, name, subpath, ref, ref_kind)
             VALUES (?1, 1, 'actions', 'checkout', NULL, 'v4', 'sideways')",
            [scan_id],
        );
        assert!(result.is_err(), "ref_kind CHECK constraint should reject 'sideways'");
    }
}
