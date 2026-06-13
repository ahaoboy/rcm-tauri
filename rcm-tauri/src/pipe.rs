use clap::{Parser, Subcommand};
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::ClientOptions;

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
