//! # Constants
//!
//! Constants used throughout the application

/// Local host IPv4 address and port
pub const LOCAL_SOCKET_ADDR_STR: &str = "127.0.0.1:4221";

/// Length of buffer for handling connections, 1 kB
pub const BUFFER_LEN: usize = 1 << 10;

pub const GET_ROOT_URI: &[u8] = b"GET / HTTP/1.1\r\n";
pub const GET_ECHO_URI: &[u8] = b"GET /echo/";
pub const GET_USER_AGENT_URI: &[u8] = b"GET /user-agent";
pub const GET_FILES_URI: &[u8] = b"GET /files/";
pub const POST_FILES_URI: &[u8] = b"POST /files/";

pub const HTTP_SUFFIX: &str = " HTTP/1.1";

pub const STATUS_200_OK: &str = "HTTP/1.1 200 OK";
pub const STATUS_201_CREATED: &str = "HTTP/1.1 201 Created";
pub const STATUS_404_NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
pub const STATUS_500_INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 Internal Server Error";

pub const COMPRESSION_SCHEME: &[u8] = b"gzip";
