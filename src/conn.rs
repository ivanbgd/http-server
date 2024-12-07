//! Connection and request handlers

use crate::cli::Args;
use crate::constants::{
    BUFFER_LEN, COMPRESSION_SCHEME, GET_ECHO_URI, GET_FILES_URI, GET_ROOT_URI, GET_USER_AGENT_URI,
    HTTP_SUFFIX, POST_FILES_URI, STATUS_200_OK, STATUS_201_CREATED, STATUS_404_NOT_FOUND,
    STATUS_500_INTERNAL_SERVER_ERROR,
};
use crate::errors::ConnectionError;
use crate::templates::{echo_html, hello_html, not_found_404_html};
use anyhow::{Context, Result};
use async_compression::tokio::write::GzipEncoder;
use httparse;
use httparse::{parse_headers, Header, Status};
use log::{trace, warn};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_connection(
    mut stream: TcpStream,
    args: &Option<Args>,
) -> Result<(), ConnectionError> {
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
        get_echo(&buf).await?
    } else if buf.starts_with(GET_USER_AGENT_URI) {
        get_user_agent(&buf)?
    } else if buf.starts_with(GET_FILES_URI) {
        get_files(&buf, &args.as_ref().expect("Expected some args").dir).await?
    } else if buf.starts_with(POST_FILES_URI) {
        post_files(&buf, &args.as_ref().expect("Expected some args").dir).await?
    } else {
        get_not_found()?
    };

    stream.write_all(&response).await?;
    stream.flush().await?;

    trace!("Stop handling request from {}", stream.peer_addr()?);

    Ok(())
}

/// `GET / HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
fn get_root() -> Result<Vec<u8>> {
    let contents = hello_html();
    let length = contents.len();
    let status_line = STATUS_200_OK;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response.as_bytes().to_vec())
}

/// `GET /echo/{text} HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
/// - `Content-Type: text/plain`
/// - `Content-Length: <text_length>`
/// - `text`
async fn get_echo(buf: &[u8]) -> Result<Vec<u8>> {
    let (_body_start, headers, _rest) = parsed_headers(buf)?;
    let mut compress = false;
    let mut compressed_header = Vec::new();
    'outer: for header in headers {
        if header.name.to_lowercase() == "Accept-Encoding".to_lowercase() {
            let header_value = header.value;
            for scheme in header_value.split(|c| c == &b',') {
                if scheme.trim_ascii() == COMPRESSION_SCHEME {
                    compress = true;
                    compressed_header.extend_from_slice(b"Content-Encoding: ");
                    compressed_header.extend_from_slice(COMPRESSION_SCHEME);
                    compressed_header.extend_from_slice(b"\r\n");
                    break 'outer;
                }
            }
        }
    }

    let rest = buf
        .strip_prefix(GET_ECHO_URI)
        .ok_or(ConnectionError::ParseError("strip echo prefix".to_string()))?;
    let line = rest.lines().next().context("error parsing line")??;
    let echo = line
        .strip_suffix(HTTP_SUFFIX)
        .ok_or(ConnectionError::ParseError("strip echo suffix".to_string()))?;
    let text = echo_html(echo);

    let mut body: Vec<u8> = Vec::new();
    if compress {
        let mut compr = GzipEncoder::new(&mut body);
        compr.write_all(text.as_ref()).await?;
        compr.flush().await?;
    } else {
        body = text.into_bytes();
    }

    let length = body.len();
    let length = format!("{}", length).into_bytes();

    let mut resp_headers: Vec<u8> = Vec::new();
    resp_headers.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
    resp_headers.extend_from_slice(&compressed_header);
    resp_headers.extend_from_slice(b"Content-Type: text/plain\r\n");
    resp_headers.extend_from_slice(b"Content-Length: ");
    resp_headers.extend_from_slice(&length);
    resp_headers.extend_from_slice(b"\r\n\r\n");

    let mut response: Vec<u8> = resp_headers;
    response.extend_from_slice(&body);

    Ok(response.to_vec())
}

/// `GET /user-agent HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
/// - `Content-Type: text/plain`
/// - `Content-Length: <user-agent_length>`
/// - `"User-Agent"'s value`
fn get_user_agent(buf: &[u8]) -> Result<Vec<u8>> {
    let (body_start, headers, _rest) = parsed_headers(buf)?;
    trace!("Request body begins at byte index {:?}.", body_start);
    let contents = headers
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
    Ok(response.as_bytes().to_vec())
}

/// `GET /files/{filename} HTTP/1.1`
///
/// - `HTTP/1.1 200 OK`
/// - `Content-Type: application/octet-stream`
/// - `Content-Length: <file_length>`
/// - `<file_contents>`
///
/// `dir` specifies the directory where the files are stored, as an absolute path.
async fn get_files(buf: &[u8], dir: &Path) -> Result<Vec<u8>> {
    let file_path = get_file_path(buf, dir)?;
    let mut status_line = STATUS_404_NOT_FOUND;
    let mut contents = Vec::new();
    if let Ok(mut file) = File::open(file_path).await {
        status_line = STATUS_200_OK;
        file.read_to_end(&mut contents).await?;
    }
    let contents = String::from_utf8_lossy(&contents);
    let length = contents.len();
    let header =
        format!("{status_line}\r\nContent-Type: application/octet-stream\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response.as_bytes().to_vec())
}

/// `POST /files/{filename} HTTP/1.1`
/// - `Content-Type: application/octet-stream`
/// - `Content-Length: <file_length>` - length of body
/// - `<file_contents>` - request body: some text
///
/// - `HTTP/1.1 201 Created`
///
/// `dir` specifies the directory where the files are stored, as an absolute path.
///
/// Creates a new file in the given directory with the following requirements:
/// - The filename equals the filename parameter in the endpoint.
/// - The file contains the contents of the request body.
async fn post_files(buf: &[u8], dir: &Path) -> Result<Vec<u8>, ConnectionError> {
    let (body_start, headers, rest) = parsed_headers(buf)?;
    let mut content_type_found = false;
    let mut content_length: usize = 0;
    for header in headers {
        if header.name.to_lowercase() == "Content-Type".to_lowercase() {
            let header_value = String::from_utf8_lossy(header.value);
            if header_value != "application/octet-stream" {
                return Err(ConnectionError::ContentTypeMissingOrWrong(
                    header_value.to_string(),
                ));
            }
            content_type_found = true;
        } else if header.name.to_lowercase() == "Content-Length".to_lowercase() {
            let len = String::from_utf8(Vec::from(header.value)).context("parse content length")?;
            content_length = len.parse()?;
            trace!(
                "Content-Length: {:?}; parsed value: {}",
                header.value,
                content_length
            );
        }
    }
    if !content_type_found {
        return Err(ConnectionError::ContentTypeMissingOrWrong(
            "missing the Content-Type header".to_string(),
        ));
    }

    trace!("Request body begins at byte index {:?}.", body_start);
    let contents = &rest[body_start..body_start + content_length];
    assert_eq!(content_length, contents.len());

    let file_path = post_file_path(buf, dir)?;
    let mut status_line = STATUS_500_INTERNAL_SERVER_ERROR;
    if let Ok(mut file) = File::create(file_path).await {
        status_line = STATUS_201_CREATED;
        file.write_all(contents).await?;
        file.flush().await?;
    };

    let contents = "".to_string();
    let length = contents.len();
    let header =
        format!("{status_line}\r\nContent-Type: application/octet-stream\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response.as_bytes().to_vec())
}

/// `GET /{non-existent} HTTP/1.1`
///
/// - `HTTP/1.1 404 Not Found`
fn get_not_found() -> Result<Vec<u8>> {
    let contents = not_found_404_html();
    let length = contents.len();
    let status_line = STATUS_404_NOT_FOUND;
    let header =
        format!("{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n");
    let response = format!("{header}{contents}");
    Ok(response.as_bytes().to_vec())
}

/// Extracts file path from an HTTP GET request and returns it
fn get_file_path(buf: &[u8], dir: &Path) -> Result<PathBuf> {
    let rest = buf
        .strip_prefix(GET_FILES_URI)
        .ok_or(ConnectionError::ParseError(
            "strip files prefix".to_string(),
        ))?;
    let line = rest.lines().next().context("error parsing line")??;
    let file_name = line
        .strip_suffix(HTTP_SUFFIX)
        .ok_or(ConnectionError::ParseError(
            "strip files suffix".to_string(),
        ))?;
    let file_path = dir.join(file_name);
    Ok(file_path)
}

/// Extracts file path from an HTTP POST request and returns it
fn post_file_path(buf: &[u8], dir: &Path) -> Result<PathBuf> {
    let rest = buf
        .strip_prefix(POST_FILES_URI)
        .ok_or(ConnectionError::ParseError(
            "strip files prefix".to_string(),
        ))?;
    let line = rest.lines().next().context("error parsing line")??;
    let file_name = line
        .strip_suffix(HTTP_SUFFIX)
        .ok_or(ConnectionError::ParseError(
            "strip files suffix".to_string(),
        ))?;
    let file_path = dir.join(file_name);
    Ok(file_path)
}

/// Returns the body starting index, a list of parsed headers and a reference
/// to the list of headers inside the request buffer, `buf`, hence, unparsed.
fn parsed_headers(buf: &[u8]) -> Result<(usize, Vec<Header>, &[u8])> {
    let line = buf
        .lines()
        .next()
        .ok_or(ConnectionError::LineParseError)??;
    trace!("{}", line);
    let rest = &buf[line.len() + 2..];
    trace!("{}", String::from_utf8_lossy(rest).replace("\r\n", " "));
    let mut headers = [httparse::EMPTY_HEADER; 8];
    while let Status::Partial = parse_headers(rest, &mut headers)? {}
    let parsed = parse_headers(rest, &mut headers)?.unwrap();
    let body_start = parsed.0;
    let headers = parsed.1.to_owned();

    Ok((body_start, headers, rest))
}
