//! # An HTTP Server

use anyhow::Result;
use codecrafters_http_server::constants::LOCAL_SOCKET_ADDR_STR;
use log::{info, warn};
use std::net::TcpListener;

fn main() -> Result<()> {
    env_logger::init();
    info!("starting the app...");

    let listener = TcpListener::bind(LOCAL_SOCKET_ADDR_STR)?;

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                info!("accepted a new connection");
            }
            Err(e) => {
                warn!("error: {}", e);
            }
        }
    }

    Ok(())
}
