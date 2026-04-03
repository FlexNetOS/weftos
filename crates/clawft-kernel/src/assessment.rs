//! Project assessment service for WeftOS kernel.
//!
//! Provides automated codebase analysis: file scanning, complexity
//! detection, >500-line warnings, TODO tracking, and optional
//! tree-sitter symbol extraction. Supports peer linking for
//! cross-project comparison.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::health::HealthStatus;
use crate::service::{ServiceType, SystemService};

// ── Report types ────────────────────────────────────────────────

/// Full assessment report produced by a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentReport {
    /// When the assessment was run.
    pub timestamp: DateTime<Utc>,
    /// Scope that was used (full, commit, ci, dependency).
    pub scope: String,
    /// Root directory that was scanned.
    pub project: String,
    /// Number of files scanned.
    pub files_scanned: usize,
    /// Aggregate summary metrics.
    pub summary: AssessmentSummary,
    /// Individual findings (warnings, issues).
    pub findings: Vec<Finding>,
}

/// Aggregate metrics from an assessment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentSummary {
    pub total_files: usize,
    pub lines_of_code: usize,
    pub rust_files: usize,
    pub typescript_files: usize,
    pub config_files: usize,
    pub doc_files: usize,
    pub dependency_files: usize,
    pub complexity_warnings: usize,
    pub coherence_score: f64,
    pub symbols_extracted: usize,
    pub avg_complexity: f64,
}

/// A single finding from the assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

/// A linked peer project for cross-project comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub name: String,
    pub location: String,
    pub linked_at: DateTime<Utc>,
    pub last_assessment: Option<AssessmentReport>,
}

/// Comparison between local and remote peer assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub local: AssessmentReport,
    pub remote_name: String,
    pub remote: AssessmentReport,
    pub shared_deps: Vec<String>,
}

// ── Service ─────────────────────────────────────────────────────

/// Project assessment service.
///
/// Scans a project directory for code quality signals, complexity
/// warnings, and structural issues. Optionally uses tree-sitter for
/// Rust symbol extraction and complexity analysis.
pub struct AssessmentService {
    started: AtomicBool,
    latest: Mutex<Option<AssessmentReport>>,
    peers: Mutex<Vec<PeerInfo>>,
}

impl AssessmentService {
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            latest: Mutex::new(None),
            peers: Mutex::new(Vec::new()),
        }
    }

    /// Run the full assessment pipeline on `project_dir`.
    ///
    /// `scope` selects what to scan:
    /// - `"full"` — all files under project_dir
    /// - `"commit"` — only files changed in the last git commit
    /// - `"ci"` — CI config files only
    /// - `"dependency"` — dependency manifests only
    ///
    /// `format` is reserved for future output formatting (currently ignored).
    pub fn run_assessment(
        &self,
        project_dir: &Path,
        scope: &str,
        _format: &str,
    ) -> Result<AssessmentReport, String> {
        let files = match scope {
            "commit" => collect_git_changed_files(project_dir)?,
            "ci" => collect_files_filtered(project_dir, |p| is_ci_file(p)),
            "dependency" => collect_files_filtered(project_dir, |p| is_dependency_file(p)),
            _ => collect_all_files(project_dir),
        };

        let mut summary = AssessmentSummary::default();
        let mut findings = Vec::new();
        #[allow(unused_mut)]
        let mut total_complexity_sum: f64 = 0.0;
        #[allow(unused_mut)]
        let mut complexity_count: usize = 0;

        for path in &files {
            let rel = path.strip_prefix(project_dir).unwrap_or(path);
            let rel_str = rel.display().to_string();

            // Classify file
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "rs" => summary.rust_files += 1,
                "ts" | "tsx" => summary.typescript_files += 1,
                "toml" | "yaml" | "yml" | "json" if is_config_file(path) => {
                    summary.config_files += 1;
                }
                "md" | "txt" | "adoc" => summary.doc_files += 1,
                _ => {}
            }
            if is_dependency_file(path) {
                summary.dependency_files += 1;
            }

            // Read content
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // skip binary / unreadable files
            };

            let line_count = content.lines().count();
            summary.lines_of_code += line_count;

            // >500 line warning
            if line_count > 500 {
                summary.complexity_warnings += 1;
                findings.push(Finding {
                    severity: "warning".into(),
                    category: "size".into(),
                    file: rel_str.clone(),
                    line: None,
                    message: format!("File has {line_count} lines (>500 limit)"),
                });
            }

            // TODO detection
            for (i, line) in content.lines().enumerate() {
                if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") {
                    findings.push(Finding {
                        severity: "info".into(),
                        category: "todo".into(),
                        file: rel_str.clone(),
                        line: Some(i + 1),
                        message: line.trim().to_string(),
                    });
                }
            }

            // Tree-sitter analysis for Rust files
            #[cfg(feature = "treesitter")]
            if ext == "rs" {
                if let Ok(tree) = clawft_plugin_treesitter::analysis::parse_source(
                    &content,
                    clawft_plugin_treesitter::types::Language::Rust,
                ) {
                    let symbols = clawft_plugin_treesitter::analysis::extract_symbols(
                        &tree,
                        &content,
                        clawft_plugin_treesitter::types::Language::Rust,
                    );
                    summary.symbols_extracted += symbols.len();

                    let metrics = clawft_plugin_treesitter::analysis::calculate_complexity(
                        &tree,
                        &content,
                        clawft_plugin_treesitter::types::Language::Rust,
                    );
                    for func in &metrics.functions {
                        total_complexity_sum += func.complexity as f64;
                        complexity_count += 1;
                        if func.complexity > 10 {
                            summary.complexity_warnings += 1;
                            findings.push(Finding {
                                severity: "warning".into(),
                                category: "complexity".into(),
                                file: rel_str.clone(),
                                line: Some(func.start_line),
                                message: format!(
                                    "Function '{}' has cyclomatic complexity {}",
                                    func.name, func.complexity
                                ),
                            });
                        }
                    }
                }
            }
        }

        summary.total_files = files.len();
        summary.avg_complexity = if complexity_count > 0 {
            total_complexity_sum / complexity_count as f64
        } else {
            0.0
        };
        // Simple coherence score: ratio of files without warnings
        let warning_count = findings.iter().filter(|f| f.severity == "warning").count();
        summary.coherence_score = if summary.total_files > 0 {
            1.0 - (warning_count as f64 / summary.total_files as f64).min(1.0)
        } else {
            1.0
        };

        let report = AssessmentReport {
            timestamp: Utc::now(),
            scope: scope.to_string(),
            project: project_dir.display().to_string(),
            files_scanned: files.len(),
            summary,
            findings,
        };

        *self.latest.lock().unwrap() = Some(report.clone());
        debug!(
            scope = scope,
            files = report.files_scanned,
            "assessment complete"
        );
        Ok(report)
    }

    /// Returns the latest assessment report, if any.
    pub fn get_latest(&self) -> Option<AssessmentReport> {
        self.latest.lock().unwrap().clone()
    }

    /// Returns all linked peers.
    pub fn list_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().clone()
    }

    /// Link a peer project for comparison.
    pub fn link_peer(&self, name: String, location: String) -> Result<(), String> {
        let mut peers = self.peers.lock().unwrap();
        if peers.iter().any(|p| p.name == name) {
            return Err(format!("peer '{name}' already linked"));
        }
        peers.push(PeerInfo {
            name,
            location,
            linked_at: Utc::now(),
            last_assessment: None,
        });
        Ok(())
    }

    /// Run assessment on a peer and compare with the latest local assessment.
    pub fn compare_with_peer(&self, peer_name: &str) -> Result<ComparisonReport, String> {
        let local = self
            .get_latest()
            .ok_or_else(|| "no local assessment available; run an assessment first".to_string())?;

        let peers = self.peers.lock().unwrap();
        let peer = peers
            .iter()
            .find(|p| p.name == peer_name)
            .ok_or_else(|| format!("peer '{peer_name}' not found"))?
            .clone();
        drop(peers);

        let peer_dir = PathBuf::from(&peer.location);
        if !peer_dir.exists() {
            return Err(format!(
                "peer directory '{}' does not exist",
                peer.location
            ));
        }

        let remote = self.run_assessment(&peer_dir, &local.scope, "json")?;

        // Update peer's last_assessment
        {
            let mut peers = self.peers.lock().unwrap();
            if let Some(p) = peers.iter_mut().find(|p| p.name == peer_name) {
                p.last_assessment = Some(remote.clone());
            }
        }

        // Find shared dependency files by name
        let local_deps: std::collections::HashSet<String> = local
            .findings
            .iter()
            .filter(|f| f.category == "dependency")
            .map(|f| f.file.clone())
            .collect();
        let remote_deps: std::collections::HashSet<String> = remote
            .findings
            .iter()
            .filter(|f| f.category == "dependency")
            .map(|f| f.file.clone())
            .collect();
        let shared_deps: Vec<String> = local_deps.intersection(&remote_deps).cloned().collect();

        Ok(ComparisonReport {
            local,
            remote_name: peer_name.to_string(),
            remote,
            shared_deps,
        })
    }
}

impl Default for AssessmentService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemService for AssessmentService {
    fn name(&self) -> &str {
        "assessment"
    }

    fn service_type(&self) -> ServiceType {
        ServiceType::Custom("assessment".into())
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.started.store(true, Ordering::Relaxed);
        tracing::info!("assessment service started");
        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.started.store(false, Ordering::Relaxed);
        tracing::info!("assessment service stopped");
        Ok(())
    }

    async fn health_check(&self) -> HealthStatus {
        if self.started.load(Ordering::Relaxed) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded("not started".into())
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────

fn collect_all_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files, &|_| true);
    files
}

fn collect_files_filtered<F: Fn(&Path) -> bool>(dir: &Path, predicate: F) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files, &predicate);
    files
}

fn walk_dir<F: Fn(&Path) -> bool>(dir: &Path, out: &mut Vec<PathBuf>, predicate: &F) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Skip hidden dirs and common noise
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
        }
        if path.is_dir() {
            walk_dir(&path, out, predicate);
        } else if predicate(&path) {
            out.push(path);
        }
    }
}

fn collect_git_changed_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("failed to run git diff: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| dir.join(l))
        .filter(|p| p.exists())
        .collect();
    Ok(files)
}

fn is_ci_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let path_str = path.display().to_string();
    name.contains("ci")
        || name.contains("CI")
        || path_str.contains(".github/workflows")
        || name == "Jenkinsfile"
        || name == ".gitlab-ci.yml"
}

fn is_dependency_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    matches!(
        name,
        "Cargo.toml"
            | "Cargo.lock"
            | "package.json"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "go.mod"
            | "go.sum"
            | "requirements.txt"
            | "Pipfile"
            | "Pipfile.lock"
    )
}

fn is_config_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Dependency files are not config files
    if is_dependency_file(path) {
        return false;
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(ext, "toml" | "yaml" | "yml" | "json")
        || name == ".editorconfig"
        || name == ".rustfmt.toml"
        || name == "clippy.toml"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Create a Rust file (>500 lines)
        let mut big = String::new();
        for i in 0..510 {
            big.push_str(&format!("// line {i}\n"));
        }
        fs::write(dir.path().join("big.rs"), &big).unwrap();

        // Create a small Rust file with a TODO
        fs::write(
            dir.path().join("small.rs"),
            "fn main() {\n    // TODO: implement\n}\n",
        )
        .unwrap();

        // Create a config file
        fs::write(dir.path().join("config.toml"), "[section]\nkey = 1\n").unwrap();

        // Create a markdown doc
        fs::write(dir.path().join("README.md"), "# Readme\n").unwrap();

        // Create a dep file
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        dir
    }

    #[test]
    fn full_assessment_scans_files() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        let report = svc
            .run_assessment(dir.path(), "full", "json")
            .unwrap();

        assert_eq!(report.scope, "full");
        assert!(report.files_scanned >= 4);
        assert!(report.summary.rust_files >= 2);
        assert!(report.summary.doc_files >= 1);
        assert!(report.summary.dependency_files >= 1);
    }

    #[test]
    fn detects_large_files() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        let report = svc.run_assessment(dir.path(), "full", "json").unwrap();

        let size_warnings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "size")
            .collect();
        assert!(!size_warnings.is_empty(), "should detect >500 line file");
    }

    #[test]
    fn detects_todos() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        let report = svc.run_assessment(dir.path(), "full", "json").unwrap();

        let todos: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "todo")
            .collect();
        assert!(!todos.is_empty(), "should detect TODO comments");
    }

    #[test]
    fn get_latest_returns_last() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        assert!(svc.get_latest().is_none());

        svc.run_assessment(dir.path(), "full", "json").unwrap();
        assert!(svc.get_latest().is_some());
    }

    #[test]
    fn peer_link_and_list() {
        let svc = AssessmentService::new();
        assert!(svc.list_peers().is_empty());

        svc.link_peer("other".into(), "/tmp/other".into()).unwrap();
        assert_eq!(svc.list_peers().len(), 1);
        assert_eq!(svc.list_peers()[0].name, "other");

        // Duplicate link fails
        assert!(svc.link_peer("other".into(), "/tmp/other2".into()).is_err());
    }

    #[test]
    fn dependency_scope_filters() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        let report = svc
            .run_assessment(dir.path(), "dependency", "json")
            .unwrap();

        // Should only scan dependency files
        assert_eq!(report.files_scanned, 1); // Cargo.toml
    }

    #[test]
    fn coherence_score_is_bounded() {
        let dir = setup_test_dir();
        let svc = AssessmentService::new();
        let report = svc.run_assessment(dir.path(), "full", "json").unwrap();

        assert!(report.summary.coherence_score >= 0.0);
        assert!(report.summary.coherence_score <= 1.0);
    }
}
