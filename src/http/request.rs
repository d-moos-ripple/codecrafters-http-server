use std::collections::HashMap;
use super::message::{HttpMessage, Startline};
use anyhow::{Context, Result};

pub type Request = HttpMessage<RequestLine>;

pub struct RequestLine {
    pub method: String,
    pub target: String,
    pub version: String
}

impl Startline for RequestLine {}

impl TryFrom<String> for HttpMessage<RequestLine> {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (raw_request_line, remaining) = value.split_once("\r\n").context("could not read request_line")?;
        let request_line = parse_requestline(raw_request_line)?;
    
        // get headers section
        let mut headers: HashMap<String, String> = HashMap::new();
        if &remaining[..4] != "\r\n" {
            let (header_section, _) = remaining.split_once("\r\n\r\n").context("could not read headers")?;
            headers = parse_headers(header_section)?;
        }
    
        // lets skip the body for now...
    
        Ok(Request::new(request_line, headers))
    }
}

fn parse_requestline(raw: &str) -> Result<RequestLine> {
    let mut split = raw.split(" ");
    let method: &str = split.next().context("could not read method")?;
    let target: &str = split.next().context("could not read target")?;
    let version: &str = split.next().context("could not read version")?;

    Ok(RequestLine {
        method: method.to_string(),
        target: target.to_string(),
        version: version.to_string()
    })
}

fn parse_headers(raw: &str) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();

    let header_lines = raw.split("\r\n");
    for header in header_lines {
        let (k, v) = header.split_once(": ").context("could not split headers correctly")?;
        headers.insert(k.to_string(), v.to_string());
    }

    Ok(headers)
}

mod tests {
    use std::collections::HashMap;
    use crate::http::request::{parse_headers, parse_requestline};


    #[test]
    fn request_line() {
        let request_line = parse_requestline("POST / HTTP/1.1").unwrap();
    
        assert_eq!(request_line.method, "POST");
        assert_eq!(request_line.target, "/");
        assert_eq!(request_line.version, "HTTP/1.1");
    }

    #[test]
    fn headers() {
        const HEADER: &str = "Header: Value\r\nFoo: Bar";
        let headers = parse_headers(HEADER).unwrap();
        assert_eq!(headers, HashMap::from([("Header".to_string(), "Value".to_string()), ("Foo".to_string(), "Bar".to_string())]));
    }
}