//! Connection facility

use crate::constants::{
    BUFFER_LEN, GET_ECHO_URI, GET_ROOT_URI, GET_USER_AGENT_URI, STATUS_200_OK, STATUS_404_NOT_FOUND,
};
use crate::errors::ConnectionError;
use crate::templates::{echo_html, hello_html, not_found_404_html};
use httparse;
use httparse::{parse_headers, Status};
use log::{trace, warn};
use std::io::BufRead;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<(), ConnectionError> {
    trace!("Start handling request from {}", stream.peer_addr()?);

    let mut buf = [0; BUFFER_LEN];
    match stream.read(&mut buf).await {
        Ok(0) => return Ok(()),
        Ok(n) => assert!(0 < n && n <= buf.len()),
        Err(err) => {
            warn!("{}", err);
            return Err(ConnectionError::from(err));
        }
    }

    let mut headers = [httparse::EMPTY_HEADER; 8];

    let (status_line, contents, length) = if buf.starts_with(GET_ROOT_URI) {
        let contents = hello_html();
        let length = contents.len();
        (STATUS_200_OK, contents, length)
    } else if buf.starts_with(GET_ECHO_URI) {
        let rest = buf
            .strip_prefix(GET_ECHO_URI)
            .ok_or(ConnectionError::ParseError("strip echo prefix".to_string()))?;
        let line = rest.lines().next().unwrap()?;
        let echo = line
            .strip_suffix(" HTTP/1.1")
            .ok_or(ConnectionError::ParseError("strip echo suffix".to_string()))?;
        let contents = echo_html(echo);
        let length = contents.len();
        (STATUS_200_OK, contents, length)
    } else if buf.starts_with(GET_USER_AGENT_URI) {
        let line = buf.lines().next().unwrap()?;
        trace!("{}", line);
        let rest = &buf[line.len() + 2..];
        trace!("{}", String::from_utf8_lossy(rest).replace("\r\n", " "));
        while let Status::Partial = parse_headers(rest, &mut headers)? {}
        let parsed = parse_headers(rest, &mut headers)?.unwrap();
        trace!("Request body begins at byte index {:?}.", parsed.0);
        let contents = parsed
            .1
            .iter()
            .find(|&&h| h.name.to_lowercase() == "User-Agent".to_lowercase())
            .ok_or(ConnectionError::UserAgentMissing)?
            .value;
        let contents = String::from_utf8_lossy(contents).to_string();
        let length = contents.len();
        (STATUS_200_OK, contents, length)
    } else {
        let contents = not_found_404_html();
        let length = contents.len();
        (STATUS_404_NOT_FOUND, contents, length)
    };

    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    trace!("Stop handling request from {}", stream.peer_addr()?);

    Ok(())
}
