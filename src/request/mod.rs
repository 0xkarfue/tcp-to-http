use std::io::{self, Read};

#[cfg(test)]
mod tests;

pub struct Request {
    pub request_line: RequestLine,
}

pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

pub fn request_from_reader(mut data: impl Read) -> Result<Request, io::Error> {
    let mut buffer = String::new();
    data.read_to_string(&mut buffer)?;

    let request_line = buffer
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing method"))?;
    // let first_line = buffer.split("\r\n");
    
    let mut request_body = request_line.splitn(3, ' ');

    let method = request_body
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing method"))?;
    let path = request_body
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request"))?;
    let version = request_body
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing version"))?;

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

    let request_line = RequestLine {
        http_version: "1.1".to_string(),
        request_target: path.to_string(),
        method: method.to_string(),
    };

    let request = Request { request_line };

    Ok(request)
}
