//! Parses a GitHub Actions workflow YAML file into the facts the
//! `workflow_parser` enricher owns (see "Enrichers" in `docs/ontology.md`):
//! whether the workflow is reusable (`on: workflow_call`), and every job's
//! step-level `uses:` (USES) and job-level `uses:` (CALLS, calling a
//! reusable workflow).
//!
//! This module only parses text into structs; it does not touch the
//! graph. Two things are deliberately deferred to week 2
//! (`docs/plan.md`):
//! - Turning a non-SHA ref's `ref_kind` from `Unknown` into `tag` or
//!   `branch` needs a GitHub API call (the `ref_resolver` enricher).
//! - Writing these facts into SQLite, which needs the open question in
//!   `docs/decisions.md` about `local`/`docker` refs settled first.

use graph::model::RefKind;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A `uses:` value, parsed but not resolved against GitHub.
///
/// `ref_kind` only reflects what's knowable without a network call: a
/// 40-hex-character ref is `Sha`, a `./...` path is `Local`, a
/// `docker://...` value is `Docker`. Everything else is `Unknown` until
/// the scanner's `ref_resolver` asks GitHub whether the ref is a tag or a
/// branch -- never guess which, per `docs/ontology.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedActionRef {
    /// Exactly as written in the workflow file, e.g.
    /// `actions/checkout@v4` or `./.github/actions/build`.
    pub written_as: String,
    pub ref_kind: RefKind,
    pub owner: Option<String>,
    pub name: Option<String>,
    pub subpath: Option<String>,
    pub git_ref: Option<String>,
}

impl ParsedActionRef {
    pub fn parse(uses: &str) -> Self {
        let written_as = uses.to_string();

        if let Some(rest) = uses.strip_prefix("./") {
            return ParsedActionRef {
                written_as,
                ref_kind: RefKind::Local,
                owner: None,
                name: None,
                subpath: Some(format!("./{rest}")),
                git_ref: None,
            };
        }
        if uses.starts_with("docker://") {
            return ParsedActionRef {
                written_as,
                ref_kind: RefKind::Docker,
                owner: None,
                name: None,
                subpath: None,
                git_ref: None,
            };
        }

        // Every remote action or reusable-workflow reference needs an
        // `@ref`; a bare path without one is malformed. Record it as
        // Unknown rather than guessing what was meant.
        let Some((path, git_ref)) = uses.split_once('@') else {
            return ParsedActionRef {
                written_as,
                ref_kind: RefKind::Unknown,
                owner: None,
                name: None,
                subpath: None,
                git_ref: None,
            };
        };

        // owner/name/subpath, where subpath may itself contain slashes
        // (e.g. a reusable workflow file's path), so cap the split at 3.
        let mut segments = path.splitn(3, '/');
        let owner = segments.next().filter(|s| !s.is_empty()).map(str::to_string);
        let name = segments.next().filter(|s| !s.is_empty()).map(str::to_string);
        let subpath = segments.next().filter(|s| !s.is_empty()).map(str::to_string);

        let is_sha = git_ref.len() == 40 && git_ref.chars().all(|c| c.is_ascii_hexdigit());
        let ref_kind = if is_sha {
            RefKind::Sha
        } else {
            // Tag vs. branch needs a GitHub API call (ref_resolver, week 2).
            RefKind::Unknown
        };

        ParsedActionRef {
            written_as,
            ref_kind,
            owner,
            name,
            subpath,
            git_ref: Some(git_ref.to_string()),
        }
    }
}

/// One job's facts: its step-level `USES` edges and, if present, the
/// job-level `uses:` that `CALLS` a reusable workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJob {
    pub job_id: String,
    pub calls: Option<ParsedActionRef>,
    pub uses: Vec<ParsedActionRef>,
}

/// One workflow file's facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorkflow {
    /// True when `on:` includes `workflow_call`, in any of its three YAML
    /// shapes (string, list, or map).
    pub is_reusable: bool,
    pub jobs: Vec<ParsedJob>,
}

/// Parse a workflow file's raw YAML text.
pub fn parse(yaml: &str) -> anyhow::Result<ParsedWorkflow> {
    let raw: RawWorkflow = serde_saphyr::from_str(yaml)?;
    let is_reusable = on_includes_workflow_call(raw.on.as_ref());
    let jobs = raw
        .jobs
        .into_iter()
        .map(|(job_id, job)| ParsedJob {
            job_id,
            calls: job.uses.as_deref().map(ParsedActionRef::parse),
            uses: job
                .steps
                .into_iter()
                .filter_map(|step| step.uses)
                .map(|uses| ParsedActionRef::parse(&uses))
                .collect(),
        })
        .collect();
    Ok(ParsedWorkflow { is_reusable, jobs })
}

/// `on:` can be a single event name, a list of names, or a map of name to
/// per-event config; `workflow_call` can appear in any of the three.
fn on_includes_workflow_call(on: Option<&serde_json::Value>) -> bool {
    match on {
        Some(serde_json::Value::String(name)) => name == "workflow_call",
        Some(serde_json::Value::Array(names)) => {
            names.iter().any(|v| v.as_str() == Some("workflow_call"))
        }
        Some(serde_json::Value::Object(events)) => events.contains_key("workflow_call"),
        _ => false,
    }
}

/// Only the fields `workflow_parser` needs; every other workflow key
/// (`runs-on`, `with`, `run`, `env`, matrix config, ...) is ignored by
/// serde's default "unknown fields are fine" behavior.
#[derive(Debug, Deserialize)]
struct RawWorkflow {
    #[serde(default)]
    on: Option<serde_json::Value>,
    #[serde(default)]
    jobs: BTreeMap<String, RawJob>,
}

#[derive(Debug, Deserialize)]
struct RawJob {
    #[serde(default)]
    uses: Option<String>,
    #[serde(default)]
    steps: Vec<RawStep>,
}

#[derive(Debug, Deserialize)]
struct RawStep {
    #[serde(default)]
    uses: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|e| panic!("reading fixture {name}: {e}"))
    }

    #[test]
    fn simple_job_parses_steps_and_classifies_sha_vs_unknown() {
        let workflow = parse(&fixture("simple-job.yml")).unwrap();
        assert!(!workflow.is_reusable);
        assert_eq!(workflow.jobs.len(), 1);

        let build = &workflow.jobs[0];
        assert_eq!(build.job_id, "build");
        assert!(build.calls.is_none(), "this job has no job-level uses:");
        // Two `uses:` steps and one `run:` step that contributes nothing.
        assert_eq!(build.uses.len(), 2);

        let checkout = &build.uses[0];
        assert_eq!(checkout.owner.as_deref(), Some("actions"));
        assert_eq!(checkout.name.as_deref(), Some("checkout"));
        assert_eq!(checkout.git_ref.as_deref(), Some("v4"));
        assert_eq!(checkout.ref_kind, RefKind::Unknown, "tag vs branch needs the API, week 2");

        let bad_action = &build.uses[1];
        assert_eq!(bad_action.owner.as_deref(), Some("DevOpsWithDavid"));
        assert_eq!(bad_action.name.as_deref(), Some("demo-bad-action"));
        assert_eq!(bad_action.ref_kind, RefKind::Sha, "40 hex chars must be classified as sha without any API call");
    }

    #[test]
    fn reusable_caller_records_a_job_level_calls_edge() {
        let workflow = parse(&fixture("reusable-caller.yml")).unwrap();
        assert!(!workflow.is_reusable, "the *caller* is not itself reusable");
        assert_eq!(workflow.jobs.len(), 1);

        let lint = &workflow.jobs[0];
        assert_eq!(lint.job_id, "lint");
        assert!(lint.uses.is_empty(), "no steps on a job that only calls a reusable workflow");
        let calls = lint.calls.as_ref().expect("job-level uses: should become `calls`");
        assert_eq!(calls.owner.as_deref(), Some("DevOpsWithDavid"));
        assert_eq!(calls.name.as_deref(), Some("demo-workflows"));
        assert_eq!(calls.subpath.as_deref(), Some(".github/workflows/lint.yml"));
        assert_eq!(calls.git_ref.as_deref(), Some("v1"));
    }

    #[test]
    fn workflow_call_in_map_form_marks_the_workflow_reusable() {
        let workflow = parse(&fixture("reusable-workflow.yml")).unwrap();
        assert!(workflow.is_reusable);
    }

    #[test]
    fn on_shapes_other_than_map_are_handled() {
        // string form
        let w = parse("on: push\njobs:\n  a:\n    steps: []\n").unwrap();
        assert!(!w.is_reusable);

        // list form that does include workflow_call
        let w = parse("on: [push, workflow_call]\njobs:\n  a:\n    steps: []\n").unwrap();
        assert!(w.is_reusable);
    }

    #[test]
    fn local_and_docker_refs_are_classified_without_owner_name() {
        let workflow = parse(&fixture("local-and-docker.yml")).unwrap();
        let build = &workflow.jobs[0];
        assert_eq!(build.uses.len(), 3);

        let local = &build.uses[0];
        assert_eq!(local.ref_kind, RefKind::Local);
        assert_eq!(local.owner, None);
        assert_eq!(local.subpath.as_deref(), Some("./.github/actions/local-build"));

        let docker = &build.uses[1];
        assert_eq!(docker.ref_kind, RefKind::Docker);
        assert_eq!(docker.owner, None);
        assert_eq!(docker.written_as, "docker://alpine:3.19");

        let branch_ref = &build.uses[2];
        assert_eq!(branch_ref.ref_kind, RefKind::Unknown, "a non-sha ref is Unknown until ref_resolver runs");
        assert_eq!(branch_ref.git_ref.as_deref(), Some("main"));
    }

    #[test]
    fn a_uses_value_without_at_ref_is_unknown_not_a_guess() {
        let parsed = ParsedActionRef::parse("actions/checkout");
        assert_eq!(parsed.ref_kind, RefKind::Unknown);
        assert_eq!(parsed.owner, None);
        assert_eq!(parsed.git_ref, None);
    }
}
