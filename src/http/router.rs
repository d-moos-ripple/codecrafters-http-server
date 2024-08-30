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

unsafe impl Sync for Router {}
unsafe impl Send for Router {}

// { } wildcard pattern
const PATTERN: &str = r"\{.*?\}";

enum RouteMatch {
    Match(Option<String>),
    NoMatch,
}

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

        // todo pass actual matched wildcard (if any)
        let callback = self
            .endpoints
            .iter()
            .map(|(p, c)| (Router::match_target(p, target), c)) // find matching endpoints
            .filter(|(m, _)| matches!(m, RouteMatch::Match(_)))
            // select the first match
            .next()
            // return only the callback
            .map(|(_, c)| c)
            // fall-back to default handler if no matching is provided
            .unwrap_or(&self.default);

        callback(request).unwrap_or(internal_server_error)
    }

    fn match_target(path: &String, target: &String) -> RouteMatch {
        let regex = Regex::new(PATTERN).expect("regex issue");
        if regex.is_match(path) {
            // our registered endpoint is a wildcard

            // create a regex pattern based on the wildcard endpoint
            let re = Regex::new(r"\{[^{}]*\}").expect("invalid regex");
            let pattern = re
                .replace_all(path.replace("/", r"\/").as_str(), r"(.*?)")
                .to_string();

            let actual_route_regex = Regex::new(pattern.as_str()).expect("invalid regex");
            if let Some(captures) = actual_route_regex.captures(target) {
                return RouteMatch::Match(Some(captures.get(1).unwrap().as_str().to_string()));
            }
        } else {
            // endpoint is static
            if path == target {
                return RouteMatch::Match(None);
            }
        }

        RouteMatch::NoMatch
    }
}
