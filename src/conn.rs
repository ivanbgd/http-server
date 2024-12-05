//! Connection facility

use crate::constants::{
    BUFFER_LEN, GET_ROOT_URI, HELLO_HTML, NOT_FOUND_404_HTML, STATUS_200_OK, STATUS_404_NOT_FOUND,
};
use crate::errors::ConnectionError;
use log::{trace, warn};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};

pub fn handle_connection(
    mut stream: impl Read + Write + Debug,
) -> anyhow::Result<(), ConnectionError> {
    trace!("start handling request {stream:?}");

    let mut buffer = vec![0; BUFFER_LEN];
    match stream.read(&mut buffer) {
        Ok(0) => return Ok(()),
        Ok(n) => assert!(0 < n && n <= buffer.len()),
        Err(err) => {
            warn!("{}", err);
            return Err(ConnectionError::from(err));
        }
    }

    let (status_line, file_name) = if buffer.starts_with(GET_ROOT_URI) {
        (STATUS_200_OK, HELLO_HTML)
    } else {
        (STATUS_404_NOT_FOUND, NOT_FOUND_404_HTML)
    };

    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    trace!("stop handling request {stream:?}");

    Ok(())
}
