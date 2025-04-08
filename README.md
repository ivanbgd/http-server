# HTTP Server

# Running the Server

- If you would like to enable the added logging functionality, first set the `RUST_LOG` environment variable.
    - `export RUST_LOG=[trace | debug | info | warn]`
- Run `./run.sh` in one terminal session, and `curl -v http://localhost:4221`
  or some other network tool in another.

# Notes

- This server uses HTTP/1.1.
- Supported compression encoding is `Content-Encoding: gzip`.
