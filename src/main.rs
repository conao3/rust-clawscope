use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "clawscope", about = "Inspect OpenClaw agent sessions", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all OpenClaw sessions with status and last active time
    Sessions,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntry {
    updated_at: i64,
}

fn format_relative_time(ms: i64) -> String {
    let now = Utc::now().timestamp_millis();
    let diff_ms = now - ms;

    if diff_ms < 0 {
        return "just now".to_string();
    }

    let diff_secs = diff_ms / 1000;
    let diff_mins = diff_secs / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_secs < 60 {
        if diff_secs <= 5 {
            "just now".to_string()
        } else {
            format!("{} seconds ago", diff_secs)
        }
    } else if diff_mins < 60 {
        if diff_mins == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", diff_mins)
        }
    } else if diff_hours < 24 {
        if diff_hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", diff_hours)
        }
    } else {
        if diff_days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", diff_days)
        }
    }
}

fn is_active(updated_at_ms: i64) -> bool {
    let now = Utc::now().timestamp_millis();
    let five_minutes_ms = 5 * 60 * 1000;
    (now - updated_at_ms) < five_minutes_ms
}

fn sessions_command() -> Result<()> {
    let sessions_path: PathBuf = dirs_path();

    let content = std::fs::read_to_string(&sessions_path)
        .with_context(|| format!("Failed to read sessions file: {}", sessions_path.display()))?;

    let sessions: HashMap<String, SessionEntry> =
        serde_json::from_str(&content).context("Failed to parse sessions.json")?;

    // Sort sessions: active first, then by updatedAt descending
    let mut entries: Vec<(&String, &SessionEntry)> = sessions.iter().collect();
    entries.sort_by(|a, b| {
        let a_active = is_active(a.1.updated_at);
        let b_active = is_active(b.1.updated_at);
        match (a_active, b_active) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.1.updated_at.cmp(&a.1.updated_at),
        }
    });

    // Compute column widths
    let session_col = entries
        .iter()
        .map(|(k, _)| k.len())
        .max()
        .unwrap_or(7)
        .max(7); // "SESSION".len()

    // Print header
    println!(
        "{:<width$}  {:<8}  {}",
        "SESSION",
        "STATUS",
        "LAST ACTIVE",
        width = session_col
    );
    println!("{}", "-".repeat(session_col + 2 + 8 + 2 + 20));

    for (key, entry) in &entries {
        let status = if is_active(entry.updated_at) {
            "active"
        } else {
            "stopped"
        };
        let last_active = format_relative_time(entry.updated_at);
        println!(
            "{:<width$}  {:<8}  {}",
            key,
            status,
            last_active,
            width = session_col
        );
    }

    Ok(())
}

fn dirs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".openclaw")
        .join("agents")
        .join("main")
        .join("sessions")
        .join("sessions.json")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sessions => sessions_command(),
    }
}
