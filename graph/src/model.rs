//! Node and edge kinds from `docs/ontology.md`, as plain enums that map to
//! the TEXT columns in the SQLite schema (see `migrations/0001_init.sql`).
//!
//! These are just the vocabulary the schema accepts. Classification logic
//! (e.g. deciding a ref is a `tag` vs a `branch`) belongs to the scanner's
//! enrichers, not this crate.

use std::fmt;
use std::str::FromStr;

/// How a ref in a `uses:` line is pinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Sha,
    Tag,
    Branch,
    Local,
    Docker,
    /// The ref could not be classified (e.g. the API call failed). Never
    /// substitute a guess for this -- see "Entities" in docs/ontology.md.
    Unknown,
}

impl fmt::Display for RefKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RefKind::Sha => "sha",
            RefKind::Tag => "tag",
            RefKind::Branch => "branch",
            RefKind::Local => "local",
            RefKind::Docker => "docker",
            RefKind::Unknown => "unknown",
        })
    }
}

impl FromStr for RefKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sha" => Ok(RefKind::Sha),
            "tag" => Ok(RefKind::Tag),
            "branch" => Ok(RefKind::Branch),
            "local" => Ok(RefKind::Local),
            "docker" => Ok(RefKind::Docker),
            "unknown" => Ok(RefKind::Unknown),
            other => Err(format!("not a RefKind: {other}")),
        }
    }
}

/// Edge kinds from `docs/ontology.md` "Relationships".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Contains,
    Uses,
    Calls,
    Includes,
    ResolvesTo,
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EdgeKind::Contains => "CONTAINS",
            EdgeKind::Uses => "USES",
            EdgeKind::Calls => "CALLS",
            EdgeKind::Includes => "INCLUDES",
            EdgeKind::ResolvesTo => "RESOLVES_TO",
        })
    }
}

impl FromStr for EdgeKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CONTAINS" => Ok(EdgeKind::Contains),
            "USES" => Ok(EdgeKind::Uses),
            "CALLS" => Ok(EdgeKind::Calls),
            "INCLUDES" => Ok(EdgeKind::Includes),
            "RESOLVES_TO" => Ok(EdgeKind::ResolvesTo),
            other => Err(format!("not an EdgeKind: {other}")),
        }
    }
}

/// Node kinds, used as the `from_kind`/`to_kind` discriminator on `edge`
/// rows instead of a separate foreign-key column per node table -- any
/// edge can target any node kind in the ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Repo,
    Workflow,
    Job,
    ActionRef,
    Commit,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NodeKind::Repo => "Repo",
            NodeKind::Workflow => "Workflow",
            NodeKind::Job => "Job",
            NodeKind::ActionRef => "ActionRef",
            NodeKind::Commit => "Commit",
        })
    }
}

impl FromStr for NodeKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Repo" => Ok(NodeKind::Repo),
            "Workflow" => Ok(NodeKind::Workflow),
            "Job" => Ok(NodeKind::Job),
            "ActionRef" => Ok(NodeKind::ActionRef),
            "Commit" => Ok(NodeKind::Commit),
            other => Err(format!("not a NodeKind: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_kind_round_trips() {
        for kind in [
            RefKind::Sha,
            RefKind::Tag,
            RefKind::Branch,
            RefKind::Local,
            RefKind::Docker,
            RefKind::Unknown,
        ] {
            assert_eq!(kind.to_string().parse::<RefKind>().unwrap(), kind);
        }
    }

    #[test]
    fn edge_kind_round_trips() {
        for kind in [
            EdgeKind::Contains,
            EdgeKind::Uses,
            EdgeKind::Calls,
            EdgeKind::Includes,
            EdgeKind::ResolvesTo,
        ] {
            assert_eq!(kind.to_string().parse::<EdgeKind>().unwrap(), kind);
        }
    }

    #[test]
    fn node_kind_round_trips() {
        for kind in [
            NodeKind::Repo,
            NodeKind::Workflow,
            NodeKind::Job,
            NodeKind::ActionRef,
            NodeKind::Commit,
        ] {
            assert_eq!(kind.to_string().parse::<NodeKind>().unwrap(), kind);
        }
    }

    #[test]
    fn unknown_string_is_rejected_not_guessed() {
        assert!("sideways".parse::<RefKind>().is_err());
    }
}
