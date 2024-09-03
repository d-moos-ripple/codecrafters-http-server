use anyhow::{Context, Result};
use clap::Parser;
use http::{
    message::{message::HttpMessage, request::Request, response::StatusLine},
    router::Router,
};
use std::io::Read;
use std::{
    borrow::Borrow,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::io::AsyncReadExt;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

mod http;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    directory: String,
}

struct ApiContext {
    dir: String,
}

impl ApiContext {
    fn new(dir: String) -> Self {
        Self { dir }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let ctx = Arc::new(Mutex::new(ApiContext::new(args.directory)));

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let context = Arc::clone(&ctx);

        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            process_socket(socket, &create_router(context)).await;
        });
    }
}

fn create_router(ctx: Arc<Mutex<ApiContext>>) -> Router {
    let mut router = Router::new(Box::new(not_found), ctx);
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
        .add("/file/{file_path}".to_string(), Box::new(handle_file))
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

fn not_found(
    _: &Request,
    _: String,
    _: &Arc<Mutex<ApiContext>>,
) -> Result<HttpMessage<StatusLine>> {
    Ok(HttpMessage::<StatusLine>::not_found())
}

fn handle_echo(
    _: &Request,
    echo: String,
    _: &Arc<Mutex<ApiContext>>,
) -> Result<HttpMessage<StatusLine>> {
    let headers = HashMap::from([
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Content-Length".to_string(), echo.len().to_string()),
    ]);

    Ok(HttpMessage::<StatusLine>::ok(
        headers,
        Some(echo.to_string()),
    ))
}

fn handle_root(
    _: &Request,
    _: String,
    _: &Arc<Mutex<ApiContext>>,
) -> Result<HttpMessage<StatusLine>> {
    Ok(HttpMessage::<StatusLine>::ok(HashMap::new(), None))
}

fn handle_useragent(
    request: &Request,
    _: String,
    _: &Arc<Mutex<ApiContext>>,
) -> Result<HttpMessage<StatusLine>> {
    let user_agent = request
        .headers
        .get("User-Agent")
        .context("User-Agent header required")?;

    let headers = HashMap::from([
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Content-Length".to_string(), user_agent.len().to_string()),
    ]);

    let mut message = HttpMessage::<StatusLine>::ok(headers, None);
    message.write(user_agent.clone());
    Ok(message)
}

fn handle_file(
    _: &Request,
    file_name: String,
    ctx: &Arc<Mutex<ApiContext>>,
) -> Result<HttpMessage<StatusLine>> {
    let locked_ctx = ctx.lock().expect("could not lock ctx");
    let file_path = Path::new(&locked_ctx.dir).join(file_name);

    // check if file exist
    let file_handle = std::fs::File::open(file_path.to_str().unwrap());
    if file_handle.is_err() {
        return Ok(HttpMessage::<StatusLine>::not_found());
    }

    let mut buffer = String::new();
    let file_content = file_handle.unwrap().read_to_string(&mut buffer).unwrap();
    let headers = HashMap::from([
        (
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        ),
        ("Content-Length".to_string(), file_content.to_string()),
    ]);

    Ok(HttpMessage::<StatusLine>::ok(headers, Some(buffer)))
}
