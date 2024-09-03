use anyhow::Result;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::ApiContext;

use super::message::{request::Request, response::Response};

type Callback = Box<dyn Fn(&Request, String, &Arc<Mutex<ApiContext>>) -> Result<Response>>;

pub struct Router {
    default: Callback,
    endpoints: HashMap<String, Callback>,
    ctx: Arc<Mutex<ApiContext>>,
}

unsafe impl Sync for Router {}
unsafe impl Send for Router {}

const PATTERN: &str = r"\{.*?\}";

enum RouteMatch {
    Match(Option<String>),
    NoMatch,
}

impl Router {
    pub fn new(default: Callback, ctx: Arc<Mutex<ApiContext>>) -> Self {
        Self {
            default,
            endpoints: HashMap::default(),
            ctx,
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
        // todo pass actual matched wildcard (if any)
        let route = self
            .endpoints
            .iter()
            .map(|(p, c)| (Router::match_target(p, target), c)) // find matching endpoints
            .filter(|(m, _)| matches!(m, RouteMatch::Match(_)))
            // select the first match
            .next();

        let (replaced_path, callback) =
            route.map_or((String::new(), &self.default), |(route_match, cb)| {
                (
                    match route_match {
                        RouteMatch::Match(m) => m.unwrap_or(String::new()),
                        RouteMatch::NoMatch => String::new(),
                    },
                    cb,
                )
            });

        callback(request, replaced_path, &self.ctx).unwrap_or(Response::internal_error())
    }

    fn match_target(path: &String, target: &String) -> RouteMatch {
        let regex = Regex::new(PATTERN).expect("regex issue");
        if regex.is_match(path) {
            // our registered endpoint is a wildcard

            // create a regex pattern based on the wildcard endpoint
            let re = Regex::new(r"\{[^{}]*\}").expect("invalid regex");
            let pattern = re
                .replace_all(path.replace("/", r"\/").as_str(), r"(.+)")
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
