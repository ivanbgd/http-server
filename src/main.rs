//! # An HTTP Server

use anyhow::Result;
use codecrafters_http_server::cli::{cli_args, Args};
use codecrafters_http_server::conn::handle_connection;
use codecrafters_http_server::constants::LOCAL_SOCKET_ADDR_STR;
use log::{info, warn};
use std::env;
use std::process::exit;
use std::sync::OnceLock;
use tokio::net::TcpListener;

static CELL: OnceLock<Option<Args>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting the server...");

    let args = cli_args(&env::args().collect::<Vec<String>>());
    let args = CELL.get_or_init(|| args);

    let listener = TcpListener::bind(LOCAL_SOCKET_ADDR_STR).await?;

    info!("Waiting for requests...");

    loop {
        let (stream, _) = listener.accept().await?;

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            // Process each socket (stream) concurrently.
            handle_connection(stream, args)
                .await
                .map_err(|e| {
                    warn!("error: {}", e);
                })
                .expect("Failed to handle connection");

            // Await the shutdown signal
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("CTRL+C received. Shutting down...");
                    exit(0);
                }
                Err(err) => {
                    // We also shut down in case of error.
                    info!("Unable to listen for the shutdown signal: {}", err);
                    exit(-1)
                }
            };
        });
    }
}
