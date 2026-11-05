mod error;
mod grammar;
mod output;

use chrono::{DateTime, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use error::{other, target, Result};
use output::{OutputFormat, Response};

#[derive(Parser)]
#[command(name = "clawscope", about = "Inspect OpenClaw agent sessions", version)]
struct Cli {
    /// Output format (text prints the session table)
    #[arg(long, global = true, help_heading = "Global Options", value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
    /// Print debug logs to stderr
    #[arg(long, global = true, help_heading = "Global Options")]
    debug: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect OpenClaw sessions
    #[command(subcommand)]
    Session(SessionCommand),
}

#[derive(Subcommand)]
enum SessionCommand {
    /// List all OpenClaw sessions with status and last active time
    #[command(name = "list-sessions")]
    List(ListSessionsArgs),
}

#[derive(Args)]
struct ListSessionsArgs {
    /// Path of the OpenClaw sessions.json (defaults to ~/.openclaw/agents/main/sessions/sessions.json)
    #[arg(long)]
    sessions_file: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntry {
    updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ListSessionsReport {
    sessions: Vec<SessionSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SessionSummary {
    session_key: String,
    status: String,
    updated_at: String,
    last_active: String,
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

fn list_sessions(sessions_path: &Path) -> Result<ListSessionsReport> {
    let content = std::fs::read_to_string(sessions_path).map_err(|error| {
        target(format!(
            "Failed to read sessions file: {}: {error}",
            sessions_path.display()
        ))
    })?;

    let sessions: HashMap<String, SessionEntry> =
        serde_json::from_str(&content).map_err(|error| {
            target(format!(
                "Failed to parse {}: {error}",
                sessions_path.display()
            ))
        })?;

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

    Ok(ListSessionsReport {
        sessions: entries
            .into_iter()
            .map(|(key, entry)| SessionSummary {
                session_key: key.clone(),
                status: if is_active(entry.updated_at) {
                    "active"
                } else {
                    "stopped"
                }
                .to_string(),
                updated_at: DateTime::<Utc>::from_timestamp_millis(entry.updated_at)
                    .map(|time| time.to_rfc3339())
                    .unwrap_or_default(),
                last_active: format_relative_time(entry.updated_at),
            })
            .collect(),
    })
}

fn render_text(report: &ListSessionsReport) -> String {
    let mut out = String::new();

    // Compute column widths
    let session_col = report
        .sessions
        .iter()
        .map(|session| session.session_key.len())
        .max()
        .unwrap_or(7)
        .max(7); // "SESSION".len()

    // Print header
    let _ = writeln!(
        out,
        "{:<width$}  {:<8}  LAST ACTIVE",
        "SESSION",
        "STATUS",
        width = session_col
    );
    let _ = writeln!(out, "{}", "-".repeat(session_col + 2 + 8 + 2 + 20));

    for session in &report.sessions {
        let _ = writeln!(
            out,
            "{:<width$}  {:<8}  {}",
            session.session_key,
            session.status,
            session.last_active,
            width = session_col
        );
    }

    out
}

fn default_sessions_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| other("HOME is not set; pass --sessions-file".to_string()))?;
    Ok(PathBuf::from(home)
        .join(".openclaw")
        .join("agents")
        .join("main")
        .join("sessions")
        .join("sessions.json"))
}

fn to_value<T: Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| other(error.to_string()))
}

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
    };
    std::process::exit(code);
}

fn run() -> Result<()> {
    let mut root = Cli::command();
    root.build();
    let argv = grammar::prepare(&root, std::env::args_os().collect())?;
    let debug = argv.iter().any(|token| token == "--debug");
    if debug {
        eprintln!("debug: argv {:?}", &argv[1..]);
    }

    let cli = match Cli::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(error) => {
            let code = if error.use_stderr() { 252 } else { 0 };
            let _ = error.print();
            std::process::exit(code);
        }
    };

    let Commands::Session(SessionCommand::List(args)) = cli.command;
    let sessions_path = match args.sessions_file {
        Some(path) => path,
        None => default_sessions_path()?,
    };
    if debug {
        eprintln!("debug: sessions file {}", sessions_path.display());
    }
    let report = list_sessions(&sessions_path)?;
    let text = render_text(&report);
    print!(
        "{}",
        output::render(
            &grammar::pascal_case("list-sessions"),
            Response {
                value: to_value(&report)?,
                text: Some(text),
            },
            cli.output
        )?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_sessions_puts_active_first_and_rejects_a_missing_file() {
        let dir = std::env::temp_dir().join(format!("clawscope-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sessions.json");
        let now = Utc::now().timestamp_millis();
        std::fs::write(
            &path,
            format!(r#"{{"agent:old":{{"updatedAt":0}},"agent:main:main":{{"updatedAt":{now}}}}}"#),
        )
        .unwrap();
        let report = list_sessions(&path).unwrap();
        let keys: Vec<_> = report
            .sessions
            .iter()
            .map(|session| (session.session_key.as_str(), session.status.as_str()))
            .collect();
        assert_eq!(
            keys,
            [("agent:main:main", "active"), ("agent:old", "stopped")]
        );
        assert_eq!(report.sessions[1].updated_at, "1970-01-01T00:00:00+00:00");
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(list_sessions(&path).err().unwrap().exit_code(), 254);
    }
}
