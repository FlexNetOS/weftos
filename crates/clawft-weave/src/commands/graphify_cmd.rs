//! `weaver graphify` subcommand implementation.
//!
//! Provides knowledge graph commands:
//! - `weaver graphify ingest <path|url>`  -- run extraction pipeline
//! - `weaver graphify query <question>`   -- semantic search against the graph
//! - `weaver graphify export <format>`    -- export graph to file
//! - `weaver graphify diff`               -- compare current vs cached graph
//! - `weaver graphify rebuild`            -- force full re-extraction
//! - `weaver graphify watch`              -- start file watcher
//! - `weaver graphify hooks install|uninstall|status` -- manage git hooks

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Knowledge graph management subcommand.
#[derive(Parser)]
#[command(about = "Knowledge graph extraction, query, and export (graphify)")]
pub struct GraphifyArgs {
    #[command(subcommand)]
    pub action: GraphifyAction,
}

/// Graphify subcommands.
#[derive(Subcommand)]
pub enum GraphifyAction {
    /// Ingest a local path or URL into the knowledge graph.
    Ingest {
        /// Path to a local file/directory or a URL to fetch.
        target: String,

        /// Output directory for ingested files.
        #[arg(short, long, default_value = "graphify-out/memory")]
        output: PathBuf,

        /// Contributor name for metadata.
        #[arg(long)]
        contributor: Option<String>,
    },

    /// Search the knowledge graph with a natural-language question.
    Query {
        /// The question or keyword search.
        question: String,

        /// Graph JSON path.
        #[arg(short, long, default_value = "graphify-out/graph.json")]
        graph: PathBuf,

        /// Traversal mode: bfs or dfs.
        #[arg(short, long, default_value = "bfs")]
        mode: String,

        /// Traversal depth (1-6).
        #[arg(short, long, default_value_t = 3)]
        depth: usize,
    },

    /// Export the knowledge graph to a file.
    Export {
        /// Export format: json, graphml, cypher, html, obsidian, wiki, svg.
        format: String,

        /// Output path (default: graphify-out/<format>).
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Graph JSON path.
        #[arg(short, long, default_value = "graphify-out/graph.json")]
        graph: PathBuf,
    },

    /// Compare the current graph against a cached/previous version.
    Diff {
        /// Path to the old graph JSON.
        #[arg(default_value = "graphify-out/graph.json.bak")]
        old: PathBuf,

        /// Path to the current graph JSON.
        #[arg(default_value = "graphify-out/graph.json")]
        current: PathBuf,
    },

    /// Force a full re-extraction of the knowledge graph.
    Rebuild {
        /// Root directory to scan.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Clear cache before rebuilding.
        #[arg(long)]
        clean: bool,
    },

    /// Start the file watcher for automatic re-extraction.
    Watch {
        /// Root directory to watch.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Debounce window in seconds.
        #[arg(short, long, default_value_t = 2.0)]
        debounce: f64,
    },

    /// Manage git hooks for automatic graph rebuilding.
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
}

/// Git hook management subcommands.
#[derive(Subcommand)]
pub enum HooksAction {
    /// Install graphify post-commit and post-checkout hooks.
    Install {
        /// Repository root (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Remove graphify hooks.
    Uninstall {
        /// Repository root (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Check hook installation status.
    Status {
        /// Repository root (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

/// Run the graphify subcommand.
pub async fn run(args: GraphifyArgs) -> anyhow::Result<()> {
    match args.action {
        GraphifyAction::Ingest { target, output, contributor } => {
            run_ingest(&target, &output, contributor.as_deref()).await
        }
        GraphifyAction::Query { question, graph, mode, depth } => {
            run_query(&question, &graph, &mode, depth).await
        }
        GraphifyAction::Export { format, output, graph } => {
            run_export(&format, output.as_deref(), &graph).await
        }
        GraphifyAction::Diff { old, current } => {
            run_diff(&old, &current).await
        }
        GraphifyAction::Rebuild { root, clean } => {
            run_rebuild(&root, clean).await
        }
        GraphifyAction::Watch { root, debounce } => {
            run_watch(&root, debounce).await
        }
        GraphifyAction::Hooks { action } => {
            run_hooks(action).await
        }
    }
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

async fn run_ingest(
    target: &str,
    output: &std::path::Path,
    contributor: Option<&str>,
) -> anyhow::Result<()> {
    // Detect if target is a URL or local path.
    if target.starts_with("http://") || target.starts_with("https://") {
        println!("Ingesting URL: {target}");
        // URL ingestion uses the graphify ingest module.
        // In production, this would use a real HTTP client.
        // For now, report the action.
        use clawft_graphify::ingest;
        let client = ingest::StubHttpClient;
        match ingest::ingest(target, output, &client, contributor) {
            Ok(result) => {
                println!("Saved {}: {}", format!("{:?}", result.url_type), result.path.display());
            }
            Err(e) => {
                eprintln!("Ingest failed: {e}");
                std::process::exit(1);
            }
        }
    } else {
        println!("Ingesting local path: {target}");
        let path = std::path::Path::new(target);
        if !path.exists() {
            anyhow::bail!("Path does not exist: {target}");
        }
        println!("Local path ingestion: would run extraction pipeline on {target}");
        println!("(Full pipeline integration pending -- use `weaver graphify rebuild` for now)");
    }
    Ok(())
}

async fn run_query(
    question: &str,
    graph_path: &std::path::Path,
    mode: &str,
    depth: usize,
) -> anyhow::Result<()> {
    if !graph_path.exists() {
        anyhow::bail!(
            "Graph file not found: {}. Run `weaver graphify rebuild` first.",
            graph_path.display()
        );
    }

    println!("Querying graph: {}", graph_path.display());
    println!("Question: {question}");
    println!("Mode: {mode}, Depth: {depth}");

    // Load graph and perform keyword search.
    let data = std::fs::read_to_string(graph_path)?;
    let json: serde_json::Value = serde_json::from_str(&data)?;

    let nodes = json["nodes"].as_array();
    let terms: Vec<String> = question.split_whitespace()
        .filter(|t| t.len() > 2)
        .map(|t| t.to_lowercase())
        .collect();

    if let Some(nodes) = nodes {
        let mut scored: Vec<(f64, &serde_json::Value)> = nodes.iter()
            .filter_map(|n| {
                let label = n["label"].as_str().unwrap_or("").to_lowercase();
                let source = n["source_file"].as_str().unwrap_or("").to_lowercase();
                let score: f64 = terms.iter()
                    .map(|t| {
                        (if label.contains(t.as_str()) { 1.0 } else { 0.0 })
                        + (if source.contains(t.as_str()) { 0.5 } else { 0.0 })
                    })
                    .sum();
                if score > 0.0 { Some((score, n)) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        if scored.is_empty() {
            println!("No matching nodes found.");
        } else {
            println!("\nMatching nodes:");
            for (score, node) in scored.iter().take(10) {
                let label = node["label"].as_str().unwrap_or("?");
                let src = node["source_file"].as_str().unwrap_or("");
                let community = node["community"].as_u64().map(|c| c.to_string()).unwrap_or_default();
                println!("  [{score:.1}] {label} (src={src}, community={community})");
            }
        }
    } else {
        println!("No nodes found in graph.");
    }

    Ok(())
}

async fn run_export(
    format: &str,
    output: Option<&std::path::Path>,
    graph_path: &std::path::Path,
) -> anyhow::Result<()> {
    if !graph_path.exists() {
        anyhow::bail!(
            "Graph file not found: {}. Run `weaver graphify rebuild` first.",
            graph_path.display()
        );
    }

    let format_lower = format.to_lowercase();
    let default_output = PathBuf::from(match format_lower.as_str() {
        "json" => "graphify-out/graph.json",
        "obsidian" => "graphify-out/obsidian",
        "wiki" => "graphify-out/wiki",
        "html" => "graphify-out/graph.html",
        "graphml" => "graphify-out/graph.graphml",
        "cypher" => "graphify-out/graph.cypher",
        "svg" => "graphify-out/graph.svg",
        _ => "graphify-out/export",
    });

    let output = output.unwrap_or(&default_output);

    println!("Exporting graph as {format} to {}", output.display());
    println!("Source: {}", graph_path.display());

    match format_lower.as_str() {
        "obsidian" | "wiki" => {
            println!("Directory export: would create {}", output.display());
        }
        _ => {
            println!("File export: would write to {}", output.display());
        }
    }

    println!("(Export pipeline integration pending)");
    Ok(())
}

async fn run_diff(
    old_path: &std::path::Path,
    current_path: &std::path::Path,
) -> anyhow::Result<()> {
    if !current_path.exists() {
        anyhow::bail!("Current graph not found: {}", current_path.display());
    }

    println!("Comparing graphs:");
    println!("  Old:     {}", old_path.display());
    println!("  Current: {}", current_path.display());

    if !old_path.exists() {
        println!("No previous graph found -- this is the first build.");
        return Ok(());
    }

    let old_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(old_path)?)?;
    let cur_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(current_path)?)?;

    let old_nodes = old_data["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
    let cur_nodes = cur_data["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
    let old_edges = old_data["links"].as_array().map(|a| a.len()).unwrap_or(0);
    let cur_edges = cur_data["links"].as_array().map(|a| a.len()).unwrap_or(0);

    let node_diff = cur_nodes as i64 - old_nodes as i64;
    let edge_diff = cur_edges as i64 - old_edges as i64;

    println!("\nGraph diff:");
    println!("  Nodes: {old_nodes} -> {cur_nodes} ({node_diff:+})");
    println!("  Edges: {old_edges} -> {cur_edges} ({edge_diff:+})");

    Ok(())
}

async fn run_rebuild(root: &std::path::Path, clean: bool) -> anyhow::Result<()> {
    println!("Rebuilding knowledge graph from: {}", root.display());
    if clean {
        println!("Clearing cache...");
        let cache_dir = root.join(".weftos").join("graphify-cache");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)?;
            println!("Cache cleared.");
        }
    }

    println!("(Full extraction pipeline integration pending)");
    println!("Would scan {} for code and document files, extract entities and relationships,", root.display());
    println!("build the graph, cluster into communities, and write graphify-out/.");
    Ok(())
}

async fn run_watch(root: &std::path::Path, debounce: f64) -> anyhow::Result<()> {
    use clawft_graphify::watch::{WatchConfig, WatchEvent};

    let config = WatchConfig {
        root: root.to_path_buf(),
        debounce_secs: debounce,
    };

    println!("Starting file watcher...");

    // Run the polling watcher (blocks).
    clawft_graphify::watch::watch_poll(&config, |event: WatchEvent| {
        println!("[graphify watch] {} file(s) changed", event.changed.len());
        if event.has_non_code {
            println!("[graphify watch] Non-code files changed -- run `weaver graphify rebuild` for full re-extraction.");
        } else {
            println!("[graphify watch] Code-only changes -- auto-rebuild would trigger here.");
        }
    }).map_err(|e| anyhow::anyhow!("Watch error: {e}"))?;

    Ok(())
}

async fn run_hooks(action: HooksAction) -> anyhow::Result<()> {
    match action {
        HooksAction::Install { path } => {
            let msg = clawft_graphify::hooks::install_hooks(&path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{msg}");
        }
        HooksAction::Uninstall { path } => {
            let msg = clawft_graphify::hooks::uninstall_hooks(&path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{msg}");
        }
        HooksAction::Status { path } => {
            let msg = clawft_graphify::hooks::hook_status(&path)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{msg}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn graphify_args_parses() {
        GraphifyArgs::command().debug_assert();
    }
}
