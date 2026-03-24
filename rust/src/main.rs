mod port;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use clap::Parser;
use port::{KillResult, PortSpec};
use std::process;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

#[derive(Parser)]
#[command(name = "kill-the-port", version, about = "Kill processes on specified ports - fast")]
struct Cli {
    /// Ports to kill (e.g. 3000 8080 or 3000-3010)
    #[arg(required = true)]
    ports: Vec<String>,

    /// Protocol to target
    #[arg(short, long, default_value = "tcp")]
    protocol: String,

    /// Only show what would be killed, don't actually kill
    #[arg(long)]
    dry_run: bool,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Send SIGTERM instead of SIGKILL (unix only, ignored on windows)
    #[arg(long)]
    graceful: bool,
}

fn main() {
    // Reset SIGPIPE to default so piping output doesn't panic
    #[cfg(unix)]
    {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    let cli = Cli::parse();

    let ports = match PortSpec::parse_all(&cli.ports) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    let signal = if cli.graceful {
        port::Signal::Term
    } else {
        port::Signal::Kill
    };

    let mut results: Vec<KillResult> = Vec::new();
    let mut had_error = false;

    for port in &ports {
        match platform::find_and_kill(*port, &cli.protocol, cli.dry_run, signal) {
            Ok(r) => results.push(r),
            Err(e) => {
                had_error = true;
                results.push(KillResult {
                    port: *port,
                    pids: vec![],
                    error: Some(e.to_string()),
                });
            }
        }
    }

    if cli.json {
        println!("{}", serde_json_minimal(&results));
    } else {
        for r in &results {
            if let Some(ref err) = r.error {
                eprintln!("Port {}: error - {}", r.port, err);
                continue;
            }
            if r.pids.is_empty() {
                eprintln!("Port {}: no process found", r.port);
                continue;
            }
            let action = if cli.dry_run { "would kill" } else { "killed" };
            let pids: Vec<String> = r.pids.iter().map(|p| p.to_string()).collect();
            println!("Port {}: {} PID {}", r.port, action, pids.join(", "));
        }
    }

    if had_error {
        process::exit(1);
    }
}

fn serde_json_minimal(results: &[KillResult]) -> String {
    let entries: Vec<String> = results
        .iter()
        .map(|r| {
            let pids: Vec<String> = r.pids.iter().map(|p| p.to_string()).collect();
            let error = match &r.error {
                Some(e) => format!("\"{}\"", e.replace('\"', "\\\"")),
                None => "null".to_string(),
            };
            format!(
                "{{\"port\":{},\"pids\":[{}],\"error\":{}}}",
                r.port,
                pids.join(","),
                error
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}
