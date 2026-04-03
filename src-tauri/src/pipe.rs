use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

pub const PIPE_NAME: &str = r"\\.\pipe\rcm_pipe_server";

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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PipePayload {
    pub x: Option<f64>,
    pub y: Option<f64>,
}

// Client logic: Sending payload cleanly through named pipe
pub async fn send_pipe_message(x: Option<f64>, y: Option<f64>) -> std::io::Result<()> {
    let mut client = ClientOptions::new().open(PIPE_NAME)?;
    let payload = PipePayload { x, y };
    let json = serde_json::to_string(&payload)?;

    // Using simple newline demarcation for multiple stream sends
    client.write_all(format!("{json}\n").as_bytes()).await?;
    Ok(())
}

// Intercepts program flow pre-Tauri GUI initialization for quick cli util modes
pub fn check_client_cli() -> bool {
    let cli = Cli::try_parse();
    if let Ok(Cli {
        command: Some(Commands::Send { x, y }),
    }) = cli
    {
        // Evaluate tokio async client directly inline to avoid long-blocking GUI instances
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

// Server logic: Async background tokio daemon
pub fn start_pipe_server(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut server_options = ServerOptions::new();
        server_options.first_pipe_instance(true);

        let mut server = match server_options.create(PIPE_NAME) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create IPC pipe server: {}", e);
                return;
            }
        };

        loop {
            // Wait for client to hook cleanly into the IPC stream
            if server.connect().await.is_ok() {
                let mut buf = vec![0; 4096];
                if let Ok(size) = server.read(&mut buf).await
                    && let Ok(msg) = std::str::from_utf8(&buf[..size])
                {
                    // Parse JSON payloads locally
                    for line in msg.lines() {
                        if let Ok(payload) = serde_json::from_str::<PipePayload>(line.trim()) {
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or(0);

                            let menu = crate::rcm::rcm().ok();

                            let event_payload = serde_json::json!({
                                "event": "ButtonRelease",
                                "button": "Right",
                                "timestamp": timestamp,
                                "menu": menu,
                                "x": payload.x,
                                "y": payload.y
                            });

                            let _ = app_handle.emit("input-event", event_payload);
                        }
                    }
                }
            }
            // Disconnect unbinds current client allowing identical instance hook next cycle
            server.disconnect().ok();
        }
    });
}
