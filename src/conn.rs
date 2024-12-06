//! Connection and request handlers

use crate::constants::{
    BUFFER_LEN, GET_ECHO_URI, GET_ROOT_URI, GET_USER_AGENT_URI, STATUS_200_OK, STATUS_404_NOT_FOUND,
};
use crate::errors::ConnectionError;
use crate::templates::{echo_html, hello_html, not_found_404_html};
use anyhow::Result;
use httparse;
use httparse::{parse_headers, Status};
use log::{trace, warn};
use std::io::BufRead;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_connection(mut stream: TcpStream) -> Result<(), ConnectionError> {
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

    let response = if buf.starts_with(GET_ROOT_URI) {
        get_root()?
    } else if buf.starts_with(GET_ECHO_URI) {
        get_echo(&buf)?
    } else if buf.starts_with(GET_USER_AGENT_URI) {
        get_user_agent(&buf)?
    } else {
        get_not_found()?
    };

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    trace!("Stop handling request from {}", stream.peer_addr()?);

    Ok(())
}

/// `GET / HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
fn get_root() -> Result<String> {
    let contents = hello_html();
    let length = contents.len();
    let status_line = STATUS_200_OK;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response)
}

/// `GET /echo/{text} HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
/// - `Content-Type: text/plain`
/// - `Content-Length: <text_length>`
/// - `text`
fn get_echo(buf: &[u8]) -> Result<String> {
    let rest = buf
        .strip_prefix(GET_ECHO_URI)
        .ok_or(ConnectionError::ParseError("strip echo prefix".to_string()))?;
    let line = rest.lines().next().unwrap()?;
    let echo = line
        .strip_suffix(" HTTP/1.1")
        .ok_or(ConnectionError::ParseError("strip echo suffix".to_string()))?;
    let contents = echo_html(echo);
    let length = contents.len();
    let status_line = STATUS_200_OK;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response)
}

/// `GET /echo/user-agent HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
/// - `Content-Type: text/plain`
/// - `Content-Length: <user-agent_length>`
/// - `"User-Agent" value`
fn get_user_agent(buf: &[u8]) -> Result<String> {
    let line = buf.lines().next().unwrap()?;
    trace!("{}", line);
    let rest = &buf[line.len() + 2..];
    trace!("{}", String::from_utf8_lossy(rest).replace("\r\n", " "));
    let mut headers = [httparse::EMPTY_HEADER; 8];
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
    let status_line = STATUS_200_OK;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response)
}

/// `GET /{non-existent} HTTP/1.1`
///
/// - `HTTP/1.1 404 Not Found`
fn get_not_found() -> Result<String> {
    let contents = not_found_404_html();
    let length = contents.len();
    let status_line = STATUS_404_NOT_FOUND;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response)
}
