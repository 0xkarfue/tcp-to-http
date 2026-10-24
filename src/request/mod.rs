use std::io::{self, Read};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Initialized,
    Done,
}

pub struct Request {
    pub request_line: RequestLine,
    state: State,
}

#[derive(Default)]
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

const BUFFER_SIZE: usize = 8;

pub fn request_from_reader(mut reader: impl Read) -> Result<Request, io::Error> {
    let mut request = Request {
        request_line: RequestLine::default(),
        state: State::Initialized,
    };

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut read_to_index = 0;

    while request.state != State::Done {
        // If the buffer is full, double its size.
        if read_to_index == buffer.len() {
            let new_size = buffer.len() * 2;
            buffer.resize(new_size, 0);
        }

        // Read into the unused part of the buffer.
        let bytes_read = reader.read(&mut buffer[read_to_index..])?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "request line incomplete",
            ));
        }

        read_to_index += bytes_read;

        // Let the parser process all currently available bytes.
        let bytes_parsed = request.parse(&buffer[..read_to_index])?;

        // Remove only the bytes successfully parsed.
        if bytes_parsed > 0 {
            buffer.copy_within(bytes_parsed..read_to_index, 0);
            read_to_index -= bytes_parsed;
        }
    }

    Ok(request)
}

impl Request {
    fn parse(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        match self.state {
            State::Initialized => {
                let (request_line, bytes_consumed) = parse_request_line(data)?;

                if let Some(request_line) = request_line {
                    self.request_line = request_line;
                    self.state = State::Done;
                }

                Ok(bytes_consumed)
            }

            State::Done => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trying to parse data in a done state",
            )),
        }
    }
}

pub fn parse_request_line(data: &[u8]) -> Result<(Option<RequestLine>, usize), io::Error> {
    let line_end = match data.windows(2).position(|window| window == b"\r\n") {
        Some(position) => position,
        None => return Ok((None, 0)),
    };

    let line = std::str::from_utf8(&data[..line_end])
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    let parts: Vec<&str> = line.split(' ').collect();

    if parts.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "request line must have exactly three parts",
        ));
    }

    let method = parts[0];
    let path = parts[1];
    let version = parts[2];

    if method.is_empty() || !method.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid HTTP method",
        ));
    }

    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "empty request target",
        ));
    }

    if version != "HTTP/1.1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported HTTP version",
        ));
    }

    Ok((
        Some(RequestLine {
            http_version: "1.1".to_string(),
            request_target: path.to_string(),
            method: method.to_string(),
        }),
        line_end + 2,
    ))
}
