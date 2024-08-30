use anyhow::{Context, Result};
use http::{message::HttpMessage, request::Request, response::StatusLine, router::Router};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpSocket, TcpStream},
};

mod http;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            process_socket(socket, &create_router()).await;
        });
    }
}

fn create_router() -> Router {
    let mut router = Router::new(Box::new(not_found));


    router
        .add("/echo/{yolo}".to_string(), Box::new(handle_echo))
        .expect("could not add endpoint");

    router
        .add("/".to_string(), Box::new(handle_root))
        .expect("could not add endpoint");

    router
        .add("/user-agent".to_string(), Box::new(handle_useragent))
        .expect("could not add endpoint");

    router
}

async fn process_socket(mut socket: TcpStream, router: &Router) {
    println!("accepted new connection");
    let mut raw_request = String::new();

    // GPT: sometimes we do not receive an EOF marker so we read it one by one...
    let mut buffer = [0; 1024];
    let mut request = Vec::new();

    loop {
        let bytes_read = socket
            .read(&mut buffer)
            .await
            .expect("could not read bytes");

        if bytes_read == 0 {
            // Connection was closed by the client.
            println!("bytes read 0");
            break;
        }

        // Append the data read to the request buffer
        request.extend_from_slice(&buffer[..bytes_read]);

        // Check if we've hit the end of the HTTP headers (double CRLF)
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    // Convert the request to a string and print it out
    if let Ok(request_string) = String::from_utf8(request) {
        println!("Received request:\n{}", request_string);
        raw_request = request_string;
    }
    // GPT

    // parsing raw request into a struct
    let request = Request::try_from(raw_request).expect("could not parse request");

    // TODO:
    // implement endpoint routing
    let response_raw = router.execute(&request.start_line.target, &request);

    // convert into raw response
    let response = Into::<String>::into(response_raw);
    println!("{:?}", response);

    socket
        .write_all(response.as_bytes())
        .await
        .expect("could not send response");
}

fn not_found(_: &Request) -> Result<HttpMessage<StatusLine>> {
    let status_line = StatusLine::new(String::from("HTTP/1.1"), 404, String::from("Not Found"));

    Ok(HttpMessage::<StatusLine>::new(
        status_line,
        HashMap::default(),
    ))
}

fn handle_echo(request: &Request) -> Result<HttpMessage<StatusLine>> {
    // cheating. :) let's improve so that we receive the path-wildcard as an argument
    let (_, input) = request.start_line.target[1..]
        .split_once("/")
        .context("could not parse the input")?;

    let status_line = StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"));
    let headers = HashMap::from([
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Content-Length".to_string(), input.len().to_string()),
    ]);

    let mut message = HttpMessage::<StatusLine>::new(status_line, headers);
    message.write(input.to_string());

    Ok(message)
}

fn handle_root(_: &Request) -> Result<HttpMessage<StatusLine>> {
    let status_line = StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"));

    Ok(HttpMessage::<StatusLine>::new(status_line, HashMap::new()))
}

fn handle_useragent(request: &Request) -> Result<HttpMessage<StatusLine>> {
    let status_line = StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"));

    let user_agent = request
        .headers
        .get("User-Agent")
        .context("User-Agent header required")?;

    let headers = HashMap::from([
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Content-Length".to_string(), user_agent.len().to_string()),
    ]);
    let mut message = HttpMessage::<StatusLine>::new(status_line, headers);
    message.write(user_agent.clone());
    Ok(message)
}
