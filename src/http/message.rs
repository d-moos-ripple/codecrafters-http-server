use std::collections::HashMap;

pub(crate) struct HttpMessage<T: Startline> {
    pub start_line: T,
    headers: HashMap<String, String>,
    body: Option<u32>
}

impl<T: Startline> HttpMessage<T> {
    pub fn new(
        start_line: T,
        headers: HashMap<String, String>
    ) -> Self {
        Self {
            start_line,
            headers,
            body: None
        }
    }
}

impl<T: Startline + Into<String>> Into<String> for HttpMessage<T>  {
    fn into(self) -> String {
        let header_string = self.headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\r\n")
        ;

        format!("{}\r\n{}\r\n{}", self.start_line.into(), header_string, String::new())
    }
}

pub(crate) trait Startline {}

mod tests {
    use std::collections::HashMap;

    use crate::http::response::StatusLine;

    use super::HttpMessage;

    #[test]
    fn ok_response() {
        let status_line = StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"));
        let response = HttpMessage::<StatusLine>::new(status_line, HashMap::new());

        assert_eq!(
            Into::<String>::into(response),
            "HTTP/1.1 200 OK\r\n\r\n"
        );
    }
}