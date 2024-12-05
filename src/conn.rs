//! Connection facility

use crate::constants::{
    BUFFER_LEN, GET_ECHO_URI, GET_ROOT_URI, STATUS_200_OK, STATUS_404_NOT_FOUND,
};
use crate::errors::ConnectionError;
use crate::templates::{echo_html, hello_html, not_found_404_html};
use log::{trace, warn};
use std::io::BufRead;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<(), ConnectionError> {
    trace!("Start handling request from {}", stream.peer_addr()?);

    let mut buffer = [0; BUFFER_LEN];
    match stream.read(&mut buffer).await {
        Ok(0) => return Ok(()),
        Ok(n) => assert!(0 < n && n <= buffer.len()),
        Err(err) => {
            warn!("{}", err);
            return Err(ConnectionError::from(err));
        }
    }

    let (status_line, contents) = if buffer.starts_with(GET_ROOT_URI) {
        (STATUS_200_OK, hello_html())
    } else if buffer.starts_with(GET_ECHO_URI) {
        let rest = buffer
            .strip_prefix(GET_ECHO_URI)
            .ok_or(ConnectionError::ParseError("strip echo prefix".to_string()))?;
        let line = rest.lines().next().unwrap()?;
        let echo = line
            .strip_suffix(" HTTP/1.1")
            .ok_or(ConnectionError::ParseError("strip echo suffix".to_string()))?;
        (STATUS_200_OK, echo_html(echo))
    } else {
        (STATUS_404_NOT_FOUND, not_found_404_html())
    };

    let length = contents.len();
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    trace!("Stop handling request from {}", stream.peer_addr()?);

    Ok(())
}
