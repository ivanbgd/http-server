//! # An HTTP Server

use anyhow::Result;
use codecrafters_http_server::conn::handle_connection;
use codecrafters_http_server::constants::LOCAL_SOCKET_ADDR_STR;
use log::{info, warn};
use std::process::exit;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("starting the server...");

    let listener = TcpListener::bind(LOCAL_SOCKET_ADDR_STR).await?;

    info!("waiting for requests...");

    loop {
        let (stream, _) = listener.accept().await?;

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            // Process each socket (stream) concurrently.
            handle_connection(stream)
                .await
                .map_err(|e| {
                    warn!("error: {}", e);
                })
                .expect("Failed to handle connection");

            // Await the shutdown signal
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("\nCTRL+C received. Shutting down...");
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
