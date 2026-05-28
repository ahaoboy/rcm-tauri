//! Named pipe IPC server — receives right-click events from rcm_com shell extension.
//! Also handles CLI client mode for sending events to the running instance.
//!
//! Adapted from src-tauri/src/pipe.rs.

use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

pub const PIPE_NAME: &str = r"\\.\pipe\rcm_pipe_server";

/// CLI argument parser for client mode.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send an input event with optional coordinates
    Send { x: Option<f64>, y: Option<f64> },
}

/// Payload sent through the named pipe.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PipePayload {
    pub x: Option<f64>,
    pub y: Option<f64>,
}

// ── Client ──────────────────────────────────────────────────────────────

/// Send a pipe message to the running RCM instance.
pub async fn send_pipe_message(x: Option<f64>, y: Option<f64>) -> std::io::Result<()> {
    let mut client = ClientOptions::new().open(PIPE_NAME)?;
    let payload = PipePayload { x, y };
    let json = serde_json::to_string(&payload)?;
    client.write_all(format!("{json}\n").as_bytes()).await?;
    Ok(())
}

/// Check if the current process was invoked as a CLI client.
/// Returns `true` if the CLI command was handled (caller should exit).
pub fn check_client_cli() -> bool {
    let cli = Cli::try_parse();
    if let Ok(Cli {
        command: Some(Commands::Send { x, y }),
    }) = cli
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = send_pipe_message(x, y).await {
                eprintln!("Failed to connect to active rcm daemon: {}", e);
            }
        });
        return true;
    }
    false
}

// ── Server ──────────────────────────────────────────────────────────────

/// Callback type for incoming pipe events.
pub type PipeCallback = Arc<dyn Fn(f64, f64) + Send + Sync>;

/// Start the named pipe server in the background.
/// Calls `on_event(x, y)` whenever a right-click event is received.
pub fn start_pipe_server(on_event: PipeCallback) {
    tokio::spawn(async move {
        let mut server_options = ServerOptions::new();
        server_options.first_pipe_instance(true);

        let mut server = match server_options.create(PIPE_NAME) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[pipe] Failed to create IPC pipe server: {}", e);
                return;
            }
        };

        loop {
            if server.connect().await.is_ok() {
                let mut buf = vec![0u8; 4096];
                if let Ok(size) = server.read(&mut buf).await
                    && let Ok(msg) = std::str::from_utf8(&buf[..size])
                {
                    for line in msg.lines() {
                        if let Ok(payload) = serde_json::from_str::<PipePayload>(line.trim()) {
                            let x = payload.x.unwrap_or(0.0);
                            let y = payload.y.unwrap_or(0.0);
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or(0);
                            println!("[pipe] received event at ({x:.0},{y:.0}) ts={timestamp}");
                            on_event(x, y);
                        }
                    }
                }
            }
            server.disconnect().ok();
        }
    });
}
