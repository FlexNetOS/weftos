//! `weaver init` — Initialize a WeftOS project in the current directory.
//!
//! Generates `weave.toml` with sensible defaults and creates the
//! `.weftos/` runtime directory.

use clap::Parser;
use std::path::Path;

/// Initialize a WeftOS project.
#[derive(Parser)]
#[command(about = "Initialize a WeftOS project (generate weave.toml, create .weftos/)")]
pub struct InitArgs {
    /// Overwrite existing weave.toml.
    #[arg(short, long)]
    pub force: bool,

    /// Project name (defaults to current directory name).
    #[arg(short, long)]
    pub name: Option<String>,

    /// Enable mesh networking in the generated config.
    #[arg(long)]
    pub mesh: bool,

    /// Enable ECC cognitive substrate.
    #[arg(long)]
    pub ecc: bool,

    /// Skip interactive prompts.
    #[arg(short, long)]
    pub yes: bool,
}

pub async fn run(args: InitArgs) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let toml_path = cwd.join("weave.toml");

    if toml_path.exists() && !args.force {
        anyhow::bail!(
            "weave.toml already exists. Use --force to overwrite."
        );
    }

    let project_name = args.name.unwrap_or_else(|| {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project")
            .to_string()
    });

    // Generate weave.toml.
    let toml_content = generate_weave_toml(&project_name, args.mesh, args.ecc);
    std::fs::write(&toml_path, &toml_content)?;
    println!("Created weave.toml");

    // Create .weftos/ runtime directory.
    let weftos_dir = cwd.join(".weftos");
    if !weftos_dir.exists() {
        std::fs::create_dir_all(weftos_dir.join("runtime"))?;
        println!("Created .weftos/runtime/");
    }

    // Create graphify output directory.
    let graphify_dir = cwd.join("graphify-out");
    if !graphify_dir.exists() {
        std::fs::create_dir_all(&graphify_dir)?;
    }

    // Add .weftos/ to .gitignore if not already present.
    ensure_gitignore(&cwd)?;

    println!();
    println!("WeftOS project '{}' initialized.", project_name);
    println!();
    println!("Next steps:");
    println!("  weaver kernel start        # start the kernel daemon");
    println!("  weaver topology extract .  # extract codebase graph");
    println!("  weaver vault enrich .      # enrich markdown files");
    println!();

    // Chain event.
    tracing::info!(
        target: "chain_event",
        source = "weave",
        kind = "project.init",
        project = project_name,
        "chain"
    );

    Ok(())
}

fn generate_weave_toml(name: &str, mesh: bool, ecc: bool) -> String {
    let mut toml = format!(
        r#"# WeftOS project configuration
# See: https://weftos.weavelogic.ai/docs/weftos/guides/configuration

[domain]
name = "{name}"

[kernel]
max_processes = 64
health_check_interval_secs = 30

[tick]
interval_ms = 50
adaptive = true

[sources.files]
root = "."
patterns = ["**/*.rs", "**/*.ts", "**/*.py", "**/*.go", "**/*.md"]
ignore = ["target", "node_modules", "dist", ".git"]
"#
    );

    if mesh {
        toml.push_str(
            r#"
[kernel.mesh]
enabled = true
transport = "tcp"
listen_addr = "0.0.0.0:9470"
discovery = false
seed_peers = []
"#,
        );
    }

    if ecc {
        toml.push_str(
            r#"
[kernel.ecc]
enabled = true
tick_interval_ms = 1000
"#,
        );
    }

    toml.push_str(
        r#"
[embedding]
provider = "local"
model = "all-MiniLM-L6-v2"
"#,
    );

    toml
}

fn ensure_gitignore(dir: &Path) -> anyhow::Result<()> {
    let gitignore = dir.join(".gitignore");
    let entries = [".weftos/", "graphify-out/"];

    if gitignore.exists() {
        let content = std::fs::read_to_string(&gitignore)?;
        let mut additions = String::new();
        for entry in entries {
            if !content.lines().any(|l| l.trim() == entry) {
                additions.push_str(entry);
                additions.push('\n');
            }
        }
        if !additions.is_empty() {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&gitignore)?;
            std::io::Write::write_all(&mut file, b"\n# WeftOS\n")?;
            std::io::Write::write_all(&mut file, additions.as_bytes())?;
            println!("Updated .gitignore");
        }
    } else {
        std::fs::write(&gitignore, format!("# WeftOS\n{}\n", entries.join("\n")))?;
        println!("Created .gitignore");
    }

    Ok(())
}
