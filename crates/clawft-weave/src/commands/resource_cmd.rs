//! `weaver resource` — resource tree management commands.

use clap::{Args, Subcommand};
use comfy_table::{Cell, Table};

use crate::client::DaemonClient;
use crate::protocol::{Request, ResourceInspectParams, ResourceNodeInfo, ResourceStatsResult};

#[derive(Args)]
pub struct ResourceArgs {
    #[command(subcommand)]
    pub command: ResourceCommand,
}

#[derive(Subcommand)]
pub enum ResourceCommand {
    /// Display the full resource tree.
    Tree,
    /// Inspect a specific resource node.
    Inspect {
        /// Resource path (e.g. "/kernel/services/cron").
        path: String,
    },
    /// Show resource tree statistics.
    Stats,
}

pub async fn run(args: ResourceArgs) -> anyhow::Result<()> {
    let mut client = DaemonClient::connect()
        .await
        .ok_or_else(|| anyhow::anyhow!("no daemon running — start with 'weaver kernel start'"))?;

    match args.command {
        ResourceCommand::Tree => {
            let resp = client.simple_call("resource.tree").await?;
            if !resp.ok {
                anyhow::bail!("{}", resp.error.unwrap_or_default());
            }
            let nodes: Vec<ResourceNodeInfo> =
                serde_json::from_value(resp.result.unwrap_or_default())?;

            let mut table = Table::new();
            table.set_header(vec!["Path", "Kind", "Children", "Hash"]);
            for n in &nodes {
                let hash_short = if n.merkle_hash.len() >= 12 {
                    format!("{}...", &n.merkle_hash[..12])
                } else {
                    n.merkle_hash.clone()
                };
                table.add_row(vec![
                    Cell::new(&n.id),
                    Cell::new(&n.kind),
                    Cell::new(n.children.len()),
                    Cell::new(hash_short),
                ]);
            }
            println!("{table}");
        }
        ResourceCommand::Inspect { path } => {
            let params = serde_json::to_value(ResourceInspectParams { path })?;
            let resp = client
                .call(Request::with_params("resource.inspect", params))
                .await?;
            if !resp.ok {
                anyhow::bail!("{}", resp.error.unwrap_or_default());
            }
            let node: ResourceNodeInfo =
                serde_json::from_value(resp.result.unwrap_or_default())?;

            println!("Resource: {}", node.id);
            println!("  Kind:       {}", node.kind);
            println!("  Parent:     {}", node.parent.as_deref().unwrap_or("(root)"));
            println!("  Children:   {}", node.children.join(", "));
            println!("  Hash:       {}", node.merkle_hash);
            if node.metadata != serde_json::json!({}) {
                println!(
                    "  Metadata:   {}",
                    serde_json::to_string_pretty(&node.metadata)?
                );
            }
        }
        ResourceCommand::Stats => {
            let resp = client.simple_call("resource.stats").await?;
            if !resp.ok {
                anyhow::bail!("{}", resp.error.unwrap_or_default());
            }
            let stats: ResourceStatsResult =
                serde_json::from_value(resp.result.unwrap_or_default())?;

            println!("Resource Tree Statistics");
            println!("  Total nodes:   {}", stats.total_nodes);
            println!("  Namespaces:    {}", stats.namespaces);
            println!("  Services:      {}", stats.services);
            println!("  Agents:        {}", stats.agents);
            println!("  Devices:       {}", stats.devices);
            println!("  Root hash:     {}...", &stats.root_hash[..16]);
        }
    }

    Ok(())
}
