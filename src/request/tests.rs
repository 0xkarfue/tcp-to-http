
use super::*;
use std::io::{self, Cursor, Read};

// Test helper: simulates a reader that returns limited chunks.
struct ChunkReader<'a> {
    data: &'a [u8],
    num_bytes_per_read: usize,
    pos: usize,
}

impl<'a> ChunkReader<'a> {
    fn new(data: &'a str, num_bytes_per_read: usize) -> Self {
        Self {
            data: data.as_bytes(),
            num_bytes_per_read,
            pos: 0,
        }
    }
}

impl Read for ChunkReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }

        let remaining = self.data.len() - self.pos;
        let count = buf
            .len()
            .min(self.num_bytes_per_read)
            .min(remaining);

        if count == 0 {
            return Ok(0);
        }

        buf[..count].copy_from_slice(&self.data[self.pos..self.pos + count]);
        self.pos += count;

        Ok(count)
    }
}

fn parse(input: &str) -> Result<Request, io::Error> {
    request_from_reader(Cursor::new(input.as_bytes()))
}

fn assert_request(input: &str, expected_target: &str) {
    let request = parse(input).unwrap();

    assert_eq!(request.request_line.method, "GET");
    assert_eq!(request.request_line.request_target, expected_target);
    assert_eq!(request.request_line.http_version, "1.1");
}

#[test]
fn test_good_get_request() {
    assert_request(
        "GET / HTTP/1.1\r\nHost: localhost:42069\r\n\r\n",
        "/",
    );
}

#[test]
fn test_get_request_with_path() {
    assert_request(
        "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n",
        "/coffee",
    );
}

#[test]
fn test_good_post_request_with_path() {
    let input = "POST /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";
    let request = parse(input).unwrap();

    assert_eq!(request.request_line.method, "POST");
    assert_eq!(request.request_line.request_target, "/coffee");
    assert_eq!(request.request_line.http_version, "1.1");
}

#[test]
fn test_invalid_number_of_parts() {
    assert!(parse("/coffee HTTP/1.1\r\n").is_err());
    assert!(parse("GET /coffee HTTP/1.1 extra\r\n").is_err());
}

#[test]
fn test_invalid_method() {
    assert!(parse("GEt /coffee HTTP/1.1\r\n").is_err());
}

#[test]
fn test_invalid_version() {
    assert!(parse("GET /coffee HTTP/2.0\r\n").is_err());
}

#[test]
fn test_empty_input() {
    assert!(parse("").is_err());
}

#[test]
fn test_incomplete_request_line() {
    let result = parse_request_line(b"GET /coffee HTTP/1.1");

    let (request_line, bytes_consumed) = result.unwrap();

    assert!(request_line.is_none());
    assert_eq!(bytes_consumed, 0);
}

#[test]
fn test_bytes_consumed() {
    let input = b"GET /coffee HTTP/1.1\r\nHost: localhost\r\n";
    let (request_line, bytes_consumed) =
        parse_request_line(input).unwrap();

    let request_line = request_line.unwrap();

    assert_eq!(request_line.method, "GET");
    assert_eq!(request_line.request_target, "/coffee");
    assert_eq!(request_line.http_version, "1.1");
    assert_eq!(bytes_consumed, b"GET /coffee HTTP/1.1\r\n".len());
}

#[test]
fn test_different_chunk_sizes() {
    let input =
        "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

    for chunk_size in [1, 3, 8, input.len()] {
        let reader = ChunkReader::new(input, chunk_size);
        let request = request_from_reader(reader).unwrap();

        assert_eq!(request.request_line.method, "GET");
        assert_eq!(request.request_line.request_target, "/");
        assert_eq!(request.request_line.http_version, "1.1");
    }
}

#[test]
fn test_get_path_with_one_byte_chunks() {
    let input =
        "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(input, 1);
    let request = request_from_reader(reader).unwrap();

    assert_eq!(request.request_line.method, "GET");
    assert_eq!(request.request_line.request_target, "/coffee");
    assert_eq!(request.request_line.http_version, "1.1");
}
