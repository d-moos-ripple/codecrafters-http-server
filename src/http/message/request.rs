use super::message::{HttpMessage, Startline, Version};
use anyhow::{bail, Context, Result};

pub type Request = HttpMessage<RequestLine>;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
}

impl Into<String> for Method {
    fn into(self) -> String {
        match self {
            Method::Get => "GET".to_string(),
            Method::Post => "POST".to_string(),
        }
    }
}

impl TryFrom<String> for Method {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            _ => bail!("invalid input"),
        }
    }
}

#[allow(dead_code)]
pub struct RequestLine {
    pub method: Method,
    pub target: String,
    pub version: Version,
}

impl Startline for RequestLine {}

impl TryFrom<String> for RequestLine {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let request_line = parse_requestline(value.as_str())?;

        Ok(request_line)
    }
}

fn parse_requestline(raw: &str) -> Result<RequestLine> {
    let mut split = raw.split(" ");
    let method: Method = split
        .next()
        .context("could not read method")?
        .to_string()
        .try_into()?;
    let target: &str = split.next().context("could not read target")?;
    let version: Version = split
        .next()
        .context("could not read version")?
        .to_string()
        .try_into()?;

    Ok(RequestLine {
        method,
        target: target.to_string(),
        version,
    })
}

#[cfg(test)]
mod tests {
    use crate::http::message::{
        message::Version,
        request::{parse_requestline, Method},
    };

    #[test]
    fn request_line() {
        let request_line = parse_requestline("GET / HTTP/1.1").unwrap();

        assert_eq!(request_line.method, Method::Get);
        assert_eq!(request_line.target, "/");
        assert_eq!(request_line.version, Version::Http1_1);
    }
}
