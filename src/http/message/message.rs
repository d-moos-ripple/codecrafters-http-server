use std::collections::HashMap;

use anyhow::{bail, Context, Result};

use super::{
    request::{Request, RequestLine},
    response::{ClientError, ServerError, Status, StatusLine, Successful},
};

pub trait Startline {}

#[derive(Clone)]
pub(crate) struct HttpMessage<T: Startline> {
    pub start_line: T,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Version {
    Http1_1,
}

impl Into<String> for Version {
    fn into(self) -> String {
        match self {
            Version::Http1_1 => String::from("HTTP/1.1"),
        }
    }
}

impl TryFrom<String> for Version {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "HTTP/1.1" => Ok(Version::Http1_1),
            _ => bail!("unknown version"),
        }
    }
}

impl<T: Startline> HttpMessage<T> {
    pub fn new(start_line: T, headers: HashMap<String, String>, body: Option<String>) -> Self {
        Self {
            start_line,
            headers,
            body,
        }
    }

    pub fn write(&mut self, body: String) {
        self.body = Some(body);
    }
}

impl<T: Startline + Into<String>> Into<String> for HttpMessage<T> {
    fn into(self) -> String {
        let mut header_string = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join("\r\n");

        if !header_string.is_empty() {
            header_string.push_str("\r\n");
        }

        format!(
            "{}\r\n{}\r\n{}",
            self.start_line.into(),
            header_string,
            self.body.unwrap_or(String::new())
        )
    }
}

impl HttpMessage<StatusLine> {
    pub fn ok(headers: HashMap<String, String>, body: Option<String>) -> Self {
        HttpMessage::<StatusLine> {
            headers,
            start_line: StatusLine::new(Version::Http1_1, Status::Successful(Successful::Ok)),
            body,
        }
    }

    pub fn created(headers: HashMap<String, String>, body: Option<String>) -> Self {
        HttpMessage::<StatusLine> {
            headers,
            start_line: StatusLine::new(Version::Http1_1, Status::Successful(Successful::Created)),
            body,
        }
    }

    pub fn not_found() -> Self {
        HttpMessage::<StatusLine> {
            headers: HashMap::new(),
            start_line: StatusLine::new(
                Version::Http1_1,
                Status::ClientError(ClientError::NotFound),
            ),
            body: None,
        }
    }
    pub fn internal_error() -> Self {
        HttpMessage::<StatusLine> {
            headers: HashMap::new(),
            start_line: StatusLine::new(
                Version::Http1_1,
                Status::ServerError(ServerError::Internal),
            ),
            body: None,
        }
    }
}

impl TryFrom<String> for HttpMessage<RequestLine> {
    type Error = anyhow::Error;

    fn try_from(value: String) -> anyhow::Result<Self, Self::Error> {
        // POST /SIUUU HTTP/1.1 \r\n
        // [Headers] \r\n
        // [BODY]

        let (raw_request_line, remaining) = value
            .split_once("\r\n")
            .context("could not read request_line")?;

        let request_line: RequestLine = raw_request_line.to_string().try_into()?;

        // get headers (and possible body) section
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body: Option<String> = None;
        if &remaining[..4] != "\r\n" {
            let (header_section, remaining) = remaining
                .split_once("\r\n\r\n")
                .context("could not read headers")?;
            headers = parse_headers(header_section)?;

            // normally we would need to deal with the content type as well, but for now let's just stick with the length
            if let Some(content_length) = headers.get("Content-Length") {
                let len = content_length.parse::<usize>()?;
                body = Some(remaining[..len].to_string());
            }
        }

        Ok(Request::new(request_line, headers, body))
    }
}

fn parse_headers(raw: &str) -> anyhow::Result<HashMap<String, String>> {
    let mut headers = HashMap::new();

    let header_lines = raw.split("\r\n");
    for header in header_lines {
        let (k, v) = header
            .split_once(": ")
            .context("could not split headers correctly")?;
        headers.insert(k.to_string(), v.to_string());
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::http::message::{
        message::{parse_headers, Version},
        request::{Method, Request},
        response::{Status, StatusLine, Successful},
    };

    use super::HttpMessage;

    #[test]
    fn ok_response() {
        let status_line = StatusLine::new(Version::Http1_1, Status::Successful(Successful::Ok));
        let response = HttpMessage::<StatusLine>::new(status_line, HashMap::new(), None);

        assert_eq!(Into::<String>::into(response), "HTTP/1.1 200 OK\r\n\r\n");
    }

    #[test]
    fn ok_response_with_headers() {
        let status_line = StatusLine::new(Version::Http1_1, Status::Successful(Successful::Ok));
        let response = HttpMessage::<StatusLine>::new(
            status_line,
            HashMap::from([("Foo".to_string(), "Bar".to_string())]),
            None,
        );

        assert_eq!(
            Into::<String>::into(response),
            "HTTP/1.1 200 OK\r\nFoo: Bar\r\n\r\n"
        );
    }

    #[test]
    fn request_with_headers_and_body() {
        let request = "POST /files/number HTTP/1.1\r\nContent-Length: 5\r\n\r\nHallo";

        let message: Request = TryFrom::<String>::try_from(request.to_string()).unwrap();

        assert_eq!(message.start_line.method, Method::Post);
        assert_eq!(message.start_line.target, "/files/number");
        assert_eq!(message.start_line.version, Version::Http1_1);

        assert_eq!(message.body, Some("Hallo".to_string()));
        assert_eq!(
            message.headers,
            HashMap::from([("Content-Length".to_string(), "5".to_string())])
        );
    }

    #[test]
    fn headers() {
        const HEADER: &str = "Header: Value\r\nFoo: Bar";
        let headers = parse_headers(HEADER).unwrap();
        assert_eq!(
            headers,
            HashMap::from([
                ("Header".to_string(), "Value".to_string()),
                ("Foo".to_string(), "Bar".to_string())
            ])
        );
    }
}
