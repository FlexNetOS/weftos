//! Export formats for the knowledge graph.

pub mod json;
pub mod obsidian;
pub mod wiki;

use std::path::Path;

use crate::model::KnowledgeGraph;
use crate::GraphifyError;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON (`node_link_data` compatible).
    Json,
    /// GraphML XML format.
    GraphMl,
    /// Neo4j Cypher text.
    Cypher,
    /// Interactive HTML visualization (requires `html-export` feature).
    Html,
    /// Obsidian vault + canvas.
    Obsidian,
    /// SVG graph rendering.
    Svg,
    /// Wikipedia-style markdown wiki.
    Wiki,
}

impl ExportFormat {
    /// File extension for this format.
    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::GraphMl => "graphml",
            Self::Cypher => "cypher",
            Self::Html => "html",
            Self::Obsidian => "md",
            Self::Svg => "svg",
            Self::Wiki => "md",
        }
    }
}

/// Export a knowledge graph to the given format and output path.
///
/// Currently only JSON is implemented; other formats will be added in
/// later phases.
pub fn export(
    kg: &KnowledgeGraph,
    format: ExportFormat,
    output: &Path,
) -> Result<(), GraphifyError> {
    match format {
        ExportFormat::Json => json::to_json(kg, output),
        _ => Err(GraphifyError::ExportError(format!(
            "Export format {:?} not yet implemented",
            format,
        ))),
    }
}
