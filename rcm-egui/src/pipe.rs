//! Named pipe IPC server for receiving external right-click events.
//!
//! Listens on a Windows named pipe for context menu trigger events
//! from the shell extension (`rcm_com`). Uses a crossbeam channel
//! to forward events to the main egui thread.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

/// Name of the named pipe for IPC with the shell extension.
pub const PIPE_NAME: &str = r"\\.\pipe\rcm_pipe_server";

// ── CLI types ───────────────────────────────────────────────────────────

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

// ── Pipe payload ────────────────────────────────────────────────────────

/// Payload received from the named pipe client (shell extension).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipePayload {
    pub x: Option<f64>,
    pub y: Option<f64>,
}

// ── Event types for main thread communication ───────────────────────────

/// Events forwarded from background threads to the main egui thread.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Show the context menu at the given screen coordinates.
    ShowContextMenu { x: f64, y: f64 },
    /// Show the context menu with full shell extension context info.
    ShowMenuWithInfo {
        x: f64,
        y: f64,
        info: rcm_com::ContextMenuInfo,
    },
    /// Hide all context menu windows.
    HideAll,
}

/// Global sender for cross-thread events.
pub static EVENT_SENDER: Mutex<Option<crossbeam_channel::Sender<AppEvent>>> = Mutex::new(None);

/// Global receiver for cross-thread events.
pub static EVENT_RECEIVER: Mutex<Option<crossbeam_channel::Receiver<AppEvent>>> = Mutex::new(None);

// ── Client logic ────────────────────────────────────────────────────────

/// Send a pipe message to the running RCM daemon.
pub async fn send_pipe_message(x: Option<f64>, y: Option<f64>) -> std::io::Result<()> {
    let mut client = ClientOptions::new().open(PIPE_NAME)?;
    let payload = PipePayload { x, y };
    let json = serde_json::to_string(&payload)?;
    client.write_all(format!("{json}\n").as_bytes()).await?;
    Ok(())
}

/// Check if the process was invoked as a CLI client (e.g., to forward
/// coordinates to the running instance). Returns `true` if handled as CLI.
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

// ── Server logic ────────────────────────────────────────────────────────

/// Start the named pipe server in a background task.
/// Incoming events are forwarded to the main thread via the event channel.
pub fn start_pipe_server() {
    // Set up the cross-thread channel
    let (tx, rx) = crossbeam_channel::unbounded::<AppEvent>();
    *EVENT_SENDER.lock().unwrap() = Some(tx);
    *EVENT_RECEIVER.lock().unwrap() = Some(rx);

    // Spawn a dedicated OS thread with its own Tokio runtime for the pipe server.
    // eframe runs on the main thread without a Tokio runtime, so we cannot use
    // tokio::spawn directly.
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime for pipe server");
        rt.block_on(async move {
            let mut server_options = ServerOptions::new();
            server_options.first_pipe_instance(true);

            let mut server = match server_options.create(PIPE_NAME) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create IPC pipe server: {}", e);
                    return;
                }
            };

            rcm_core::log::info("pipe", "IPC pipe server started");

            loop {
                // Wait for client connection
                if server.connect().await.is_ok() {
                    let mut buf = vec![0u8; 4096];
                    if let Ok(size) = server.read(&mut buf).await
                        && let Ok(msg) = std::str::from_utf8(&buf[..size])
                    {
                        // Parse JSON payloads line by line
                        for line in msg.lines() {
                            if let Ok(PipePayload { x, y }) =
                                serde_json::from_str::<PipePayload>(line.trim())
                            {
                                rcm_core::log::info(
                                    "pipe",
                                    &format!("received event at ({:?}, {:?})", x, y),
                                );

                                // Forward to main thread
                                if let Some(tx) = EVENT_SENDER.lock().unwrap().as_ref() {
                                    if let (Some(x), Some(y)) = (x, y) {
                                        let _ = tx.send(AppEvent::ShowContextMenu { x, y });
                                    }
                                }
                            }
                        }
                    }

                    // Disconnect and wait for next client
                    server.disconnect().ok();
                }
            }
        });
    });
}
