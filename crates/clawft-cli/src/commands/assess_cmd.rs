//! `weft assess` — SOP assessment workflow.
//!
//! Run continuous assessment against a codebase to maintain the WeftOS
//! knowledge graph. Supports multiple scopes (full, commit, CI, dependency)
//! and output formats (table, JSON, GitHub annotations).
//!
//! # Usage
//!
//! ```bash
//! weft assess                          # full assessment, table output
//! weft assess run --scope commit       # only files in last commit
//! weft assess run --scope ci --format github-annotations
//! weft assess status                   # show last assessment results
//! weft assess init                     # initialize .weftos/ assessment config
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand, ValueEnum};

/// Arguments for `weft assess`.
#[derive(Args)]
pub struct AssessArgs {
    #[command(subcommand)]
    pub action: Option<AssessAction>,

    /// Assessment scope (when running without a subcommand).
    #[arg(short, long, default_value = "full")]
    pub scope: AssessScope,

    /// Output format.
    #[arg(short, long, default_value = "table")]
    pub format: AssessFormat,

    /// Project directory to assess (defaults to current directory).
    #[arg(short, long)]
    pub dir: Option<String>,
}

/// Subcommands for `weft assess`.
#[derive(Subcommand)]
pub enum AssessAction {
    /// Run an assessment (default if no subcommand given).
    Run {
        /// Assessment scope.
        #[arg(short, long, default_value = "full")]
        scope: AssessScope,

        /// Output format.
        #[arg(short, long, default_value = "table")]
        format: AssessFormat,

        /// Project directory to assess.
        #[arg(short, long)]
        dir: Option<String>,

        /// PR number for github-pr format.
        #[arg(long)]
        pr_number: Option<u64>,
    },

    /// Show status of the last assessment.
    Status {
        /// Project directory.
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Initialize assessment configuration in .weftos/.
    Init {
        /// Project directory.
        #[arg(short, long)]
        dir: Option<String>,

        /// Overwrite existing config.
        #[arg(long)]
        force: bool,
    },
}

/// Assessment scope — what to scan.
#[derive(Clone, ValueEnum)]
pub enum AssessScope {
    /// Full rescan of all files matching configured patterns.
    Full,
    /// Only files changed in the last git commit.
    Commit,
    /// All files changed in the current PR/push (CI mode).
    Ci,
    /// Only dependency manifests (Cargo.toml, package.json, etc.).
    Dependency,
}

impl std::fmt::Display for AssessScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Commit => write!(f, "commit"),
            Self::Ci => write!(f, "ci"),
            Self::Dependency => write!(f, "dependency"),
        }
    }
}

/// Output format for assessment results.
#[derive(Clone, ValueEnum)]
pub enum AssessFormat {
    /// Human-readable summary table.
    Table,
    /// Machine-readable JSON.
    Json,
    /// GitHub Actions annotation format.
    GithubAnnotations,
}

impl std::fmt::Display for AssessFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
            Self::GithubAnnotations => write!(f, "github-annotations"),
        }
    }
}

/// Run the assess command.
pub fn run(args: AssessArgs) -> anyhow::Result<()> {
    match args.action {
        Some(AssessAction::Run {
            scope,
            format,
            dir,
            pr_number,
        }) => run_assessment(&scope, &format, dir.as_deref(), pr_number),
        Some(AssessAction::Status { dir }) => run_status(dir.as_deref()),
        Some(AssessAction::Init { dir, force }) => run_init(dir.as_deref(), force),
        // No subcommand — run assessment with top-level args.
        None => run_assessment(&args.scope, &args.format, args.dir.as_deref(), None),
    }
}

// ---------------------------------------------------------------------------
// Assessment runner
// ---------------------------------------------------------------------------

fn resolve_project_dir(dir: Option<&str>) -> PathBuf {
    dir.map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn run_assessment(
    scope: &AssessScope,
    format: &AssessFormat,
    dir: Option<&str>,
    _pr_number: Option<u64>,
) -> anyhow::Result<()> {
    let project = resolve_project_dir(dir);
    let weftos_dir = project.join(".weftos");

    if !weftos_dir.exists() {
        eprintln!("No .weftos/ directory found in {}", project.display());
        eprintln!("Run `weft assess init` to initialize assessment configuration.");
        std::process::exit(1);
    }

    // Determine files to scan based on scope
    let files = match scope {
        AssessScope::Full => collect_all_files(&project)?,
        AssessScope::Commit => collect_commit_files(&project)?,
        AssessScope::Ci => collect_ci_files(&project)?,
        AssessScope::Dependency => collect_dependency_files(&project)?,
    };

    // Run the assessment pipeline: SCOPE -> SCAN -> ANALYZE -> REPORT
    let report = assess_files(&project, &files, scope)?;

    // Output in requested format
    match format {
        AssessFormat::Table => print_table_report(&report),
        AssessFormat::Json => print_json_report(&report)?,
        AssessFormat::GithubAnnotations => print_github_annotations(&report),
    }

    // Write latest assessment to .weftos/artifacts/
    let artifacts_dir = weftos_dir.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir)?;
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(
        artifacts_dir.join("assessment-latest.json"),
        json.as_bytes(),
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// File collection by scope
// ---------------------------------------------------------------------------

fn collect_all_files(project: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let patterns = ["rs", "ts", "tsx", "js", "json", "toml", "md", "mdx"];
    collect_files_recursive(project, &patterns, &mut files);
    Ok(files)
}

fn collect_commit_files(project: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD~1", "HEAD"])
        .current_dir(project)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| project.join(l))
        .filter(|p| p.exists())
        .collect())
}

fn collect_ci_files(project: &Path) -> anyhow::Result<Vec<PathBuf>> {
    // Try to find the merge base with main/master
    let base_branch = if Command::new("git")
        .args(["rev-parse", "--verify", "origin/main"])
        .current_dir(project)
        .output()
        .is_ok_and(|o| o.status.success())
    {
        "origin/main"
    } else {
        "origin/master"
    };

    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{base_branch}...HEAD")])
        .current_dir(project)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| project.join(l))
        .filter(|p| p.exists())
        .collect())
}

fn collect_dependency_files(project: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let dep_files = [
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
    ];
    Ok(dep_files
        .iter()
        .map(|f| project.join(f))
        .filter(|p| p.exists())
        .collect())
}

fn collect_files_recursive(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let skip = [
        "node_modules",
        "target",
        ".git",
        ".next",
        ".weftos",
        ".claude",
        ".planning",
    ];

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if !skip.contains(&name_str.as_ref()) {
                collect_files_recursive(&path, extensions, out);
            }
        } else if let Some(ext) = path.extension() {
            if extensions.contains(&ext.to_string_lossy().as_ref()) {
                out.push(path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Assessment pipeline
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct AssessmentReport {
    timestamp: String,
    scope: String,
    project: String,
    files_scanned: usize,
    summary: AssessmentSummary,
    findings: Vec<Finding>,
}

#[derive(serde::Serialize)]
struct AssessmentSummary {
    total_files: usize,
    lines_of_code: usize,
    rust_files: usize,
    typescript_files: usize,
    config_files: usize,
    doc_files: usize,
    dependency_files: usize,
    complexity_warnings: usize,
    coherence_score: f64,
}

#[derive(serde::Serialize)]
struct Finding {
    severity: String,
    category: String,
    file: String,
    line: Option<usize>,
    message: String,
}

fn assess_files(
    project: &Path,
    files: &[PathBuf],
    scope: &AssessScope,
) -> anyhow::Result<AssessmentReport> {
    let mut summary = AssessmentSummary {
        total_files: files.len(),
        lines_of_code: 0,
        rust_files: 0,
        typescript_files: 0,
        config_files: 0,
        doc_files: 0,
        dependency_files: 0,
        complexity_warnings: 0,
        coherence_score: 0.0,
    };

    let mut findings = Vec::new();

    for file in files {
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        // Count lines
        if let Ok(content) = std::fs::read_to_string(file) {
            let line_count = content.lines().count();
            summary.lines_of_code += line_count;

            // Categorize
            match ext.as_str() {
                "rs" => {
                    summary.rust_files += 1;
                    // Check for large files
                    if line_count > 500 {
                        findings.push(Finding {
                            severity: "medium".into(),
                            category: "complexity".into(),
                            file: file.strip_prefix(project).unwrap_or(file).display().to_string(),
                            line: None,
                            message: format!("{line_count} lines — consider splitting (target: <500)"),
                        });
                        summary.complexity_warnings += 1;
                    }
                    // Check for TODO/FIXME
                    for (i, line) in content.lines().enumerate() {
                        if line.contains("TODO") || line.contains("FIXME") {
                            findings.push(Finding {
                                severity: "info".into(),
                                category: "technical-debt".into(),
                                file: file.strip_prefix(project).unwrap_or(file).display().to_string(),
                                line: Some(i + 1),
                                message: line.trim().to_string(),
                            });
                        }
                    }
                }
                "ts" | "tsx" | "js" | "jsx" => {
                    summary.typescript_files += 1;
                    if line_count > 500 {
                        findings.push(Finding {
                            severity: "medium".into(),
                            category: "complexity".into(),
                            file: file.strip_prefix(project).unwrap_or(file).display().to_string(),
                            line: None,
                            message: format!("{line_count} lines — consider splitting"),
                        });
                        summary.complexity_warnings += 1;
                    }
                }
                "toml" | "json" => {
                    if file
                        .file_name()
                        .is_some_and(|n| {
                            let s = n.to_string_lossy();
                            s.contains("Cargo") || s.contains("package")
                        })
                    {
                        summary.dependency_files += 1;
                    } else {
                        summary.config_files += 1;
                    }
                }
                "md" | "mdx" => {
                    summary.doc_files += 1;
                }
                _ => {}
            }
        }
    }

    // Coherence score: ratio of documented modules to total modules
    let documented = summary.doc_files as f64;
    let code = (summary.rust_files + summary.typescript_files).max(1) as f64;
    summary.coherence_score = (documented / code * 100.0).min(100.0);

    Ok(AssessmentReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        scope: scope.to_string(),
        project: project.display().to_string(),
        files_scanned: files.len(),
        summary,
        findings,
    })
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

fn print_table_report(report: &AssessmentReport) {
    println!("WeftOS Assessment Report");
    println!("========================");
    println!("  Timestamp:    {}", report.timestamp);
    println!("  Scope:        {}", report.scope);
    println!("  Project:      {}", report.project);
    println!();
    println!("Summary");
    println!("-------");
    println!("  Files scanned:      {}", report.summary.total_files);
    println!("  Lines of code:      {}", report.summary.lines_of_code);
    println!("  Rust files:         {}", report.summary.rust_files);
    println!("  TypeScript files:   {}", report.summary.typescript_files);
    println!("  Config files:       {}", report.summary.config_files);
    println!("  Doc files:          {}", report.summary.doc_files);
    println!("  Dependency files:   {}", report.summary.dependency_files);
    println!(
        "  Coherence score:    {:.1}%",
        report.summary.coherence_score
    );
    println!(
        "  Complexity warns:   {}",
        report.summary.complexity_warnings
    );

    if !report.findings.is_empty() {
        println!();
        println!("Findings ({} total)", report.findings.len());
        println!("---------");
        for finding in &report.findings {
            let loc = finding
                .line
                .map(|l| format!(":{l}"))
                .unwrap_or_default();
            println!(
                "  [{:>8}] {}{} — {}",
                finding.severity, finding.file, loc, finding.message
            );
        }
    }

    println!();
    println!("Assessment saved to .weftos/artifacts/assessment-latest.json");
}

fn print_json_report(report: &AssessmentReport) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

fn print_github_annotations(report: &AssessmentReport) {
    for finding in &report.findings {
        let level = match finding.severity.as_str() {
            "critical" | "high" => "error",
            "medium" => "warning",
            _ => "notice",
        };
        let line = finding.line.unwrap_or(1);
        println!(
            "::{level} file={},line={line}::{}",
            finding.file, finding.message
        );
    }
}

// ---------------------------------------------------------------------------
// Status + Init
// ---------------------------------------------------------------------------

fn run_status(dir: Option<&str>) -> anyhow::Result<()> {
    let project = resolve_project_dir(dir);
    let latest = project.join(".weftos/artifacts/assessment-latest.json");

    if !latest.exists() {
        println!("No assessment results found.");
        println!("Run `weft assess` to perform an assessment.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&latest)?;
    let report: serde_json::Value = serde_json::from_str(&content)?;

    println!("Last Assessment");
    println!("===============");
    if let Some(ts) = report.get("timestamp").and_then(|v| v.as_str()) {
        println!("  Timestamp: {ts}");
    }
    if let Some(scope) = report.get("scope").and_then(|v| v.as_str()) {
        println!("  Scope:     {scope}");
    }
    if let Some(n) = report.get("files_scanned").and_then(|v| v.as_u64()) {
        println!("  Files:     {n}");
    }
    if let Some(summary) = report.get("summary") {
        if let Some(loc) = summary.get("lines_of_code").and_then(|v| v.as_u64()) {
            println!("  LOC:       {loc}");
        }
        if let Some(cs) = summary.get("coherence_score").and_then(|v| v.as_f64()) {
            println!("  Coherence: {cs:.1}%");
        }
    }
    if let Some(findings) = report.get("findings").and_then(|v| v.as_array()) {
        println!("  Findings:  {}", findings.len());
    }

    Ok(())
}

fn run_init(dir: Option<&str>, force: bool) -> anyhow::Result<()> {
    let project = resolve_project_dir(dir);
    let weftos_dir = project.join(".weftos");
    let config_path = weftos_dir.join("weave.toml");

    if config_path.exists() && !force {
        println!(
            ".weftos/weave.toml already exists at {}",
            config_path.display()
        );
        println!("Use --force to overwrite.");
        return Ok(());
    }

    std::fs::create_dir_all(weftos_dir.join("artifacts"))?;
    std::fs::create_dir_all(weftos_dir.join("memory"))?;

    let config = r#"# WeftOS Assessment Configuration
# See: https://weftos.weavelogic.ai/docs/weftos/guides/assessment

[assessment]
version = 1

[assessment.sources.files]
patterns = ["**/*.rs", "**/*.ts", "**/*.tsx", "**/*.json"]
exclude = ["node_modules/**", "target/**", ".weftos/**", ".git/**"]

[assessment.triggers.filesystem]
enabled = false
debounce_ms = 2000

[assessment.triggers.scheduled]
enabled = false
cron = "0 2 * * *"
scope = "full"

[assessment.reporting]
default_format = "table"
save_artifacts = true
"#;

    std::fs::write(&config_path, config)?;

    println!("Initialized WeftOS assessment at {}", weftos_dir.display());
    println!("  Created: .weftos/weave.toml");
    println!("  Created: .weftos/artifacts/");
    println!("  Created: .weftos/memory/");
    println!();
    println!("Next steps:");
    println!("  weft assess              # run your first assessment");
    println!("  weft assess status       # view results");

    Ok(())
}
