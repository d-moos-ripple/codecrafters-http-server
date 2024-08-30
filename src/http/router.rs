use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

use super::{
    message::HttpMessage,
    request::Request,
    response::{Response, StatusLine},
};

type Callback = Box<dyn Fn(&Request) -> Result<Response>>;

pub struct Router {
    default: Callback,
    endpoints: HashMap<String, Callback>,
}

// { } wildcard pattern
const PATTERN: &str = r"\{.*?\}";

impl Router {
    pub fn new(default: Callback) -> Self {
        Self {
            default,
            endpoints: HashMap::default(),
        }
    }

    pub fn add(&mut self, endpoint: String, handler: Callback) -> Result<()> {
        if self.endpoints.contains_key(&endpoint) {
            anyhow::bail!("endpoint already registered");
        }

        // will never return Some as we pre-check above
        self.endpoints.insert(endpoint, handler);

        Ok(())
    }

    // executes a request
    // if no sufficient target is found, default will be executed
    // if any handler fails, an internal server error is returned
    pub fn execute(&self, target: &String, request: &Request) -> Response {
        let internal_server_error = HttpMessage::<StatusLine>::new(
            StatusLine::new(
                "HTTP/1.1".to_string(),
                500,
                "Internal Server Error".to_string(),
            ),
            HashMap::default(),
        );

        let callback = self
            .endpoints
            .iter()
            // find matching endpoints
            .filter(|(p, _)| Router::is_match(p, target))
            // select the first match
            .next()
            // return only the callback
            .map(|(_, c)| c)
            // fall-back to default handler if no matching is provided
            .unwrap_or(&self.default);

        callback(request).unwrap_or(internal_server_error)
    }

    fn is_match(path: &String, target: &String) -> bool {
        let regex = Regex::new(PATTERN).expect("regex issue");
        if regex.is_match(path) {
            // our registered endpoint is a wildcard

            // create a regex pattern based on the wildcard endpoint
            let re = Regex::new(r"\{[^{}]*\}").expect("invalid regex");
            let pattern = re
                .replace_all(path.replace("/", r"\/").as_str(), r"(.*?)")
                .to_string();
            if Regex::new(pattern.as_str())
                .expect("not a valid pattern")
                .is_match(target)
            {
                return true;
            }
        } else {
            // endpoint is static
            if path == target {
                return true;
            }
        }

        false
    }
}
