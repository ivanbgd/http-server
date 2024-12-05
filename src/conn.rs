//! Connection facility

use crate::constants::{
    BUFFER_LEN, GET_ROOT_URI, HELLO_HTML, NOT_FOUND_404_HTML, STATUS_200_OK, STATUS_404_NOT_FOUND,
};
use crate::errors::ConnectionError;
use log::{trace, warn};
use std::fmt::Debug;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn handle_connection(
    mut stream: impl AsyncRead + AsyncWrite + Unpin + Debug,
) -> anyhow::Result<(), ConnectionError> {
    trace!("start handling request {stream:?}");

    let mut buffer = vec![0; BUFFER_LEN];
    match stream.read(&mut buffer).await {
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

    let mut file = File::open(file_name).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    trace!("stop handling request {stream:?}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::conn::handle_connection;
    use crate::constants::{GET_ROOT_URI, HELLO_HTML, STATUS_200_OK};
    use crate::mocks::MockTcpStream;

    #[tokio::test]
    async fn test_handle_connection() {
        let input_bytes = GET_ROOT_URI;
        let mut contents = vec![0u8; 1024];
        contents[..input_bytes.len()].clone_from_slice(input_bytes);

        let mut mock_tcp_stream = MockTcpStream {
            read_data: contents,
            write_data: Vec::new(),
        };

        handle_connection(&mut mock_tcp_stream).await.unwrap();

        let expected_status = STATUS_200_OK;
        let expected_contents = fs::read_to_string(HELLO_HTML).unwrap();
        let expected_length = expected_contents.len();
        let expected_response = format!(
            "{expected_status}\r\nContent-Length: {expected_length}\r\n\r\n{expected_contents}"
        );

        assert!(mock_tcp_stream
            .write_data
            .starts_with(expected_response.as_bytes()));
    }
}
