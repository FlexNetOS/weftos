//! Complexity analyzer — detects large files and TODO/FIXME/HACK markers.

use std::path::{Path, PathBuf};

use crate::assessment::{analyzer::AnalysisContext, Finding};
use crate::assessment::analyzer::Analyzer;

/// Analyzer that flags files exceeding 500 lines and tracks TODO markers.
pub struct ComplexityAnalyzer;

impl Analyzer for ComplexityAnalyzer {
    fn id(&self) -> &str {
        "complexity"
    }

    fn name(&self) -> &str {
        "Complexity Analyzer"
    }

    fn categories(&self) -> &[&str] {
        &["size", "todo"]
    }

    fn analyze(&self, project: &Path, files: &[PathBuf], _context: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for path in files {
            let rel = path.strip_prefix(project).unwrap_or(path);
            let rel_str = rel.display().to_string();

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let line_count = content.lines().count();

            // >500 line warning
            if line_count > 500 {
                findings.push(Finding {
                    severity: "warning".into(),
                    category: "size".into(),
                    file: rel_str.clone(),
                    line: None,
                    message: format!("File has {line_count} lines (>500 limit)"),
                });
            }

            // TODO / FIXME / HACK detection
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
        }

        findings
    }
}
