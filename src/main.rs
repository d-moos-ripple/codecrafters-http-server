use std::{collections::HashMap, io::{Read, Write}, net::TcpListener};
use bytes::buf;
use http::{message::HttpMessage, request::Request, response::StatusLine};

mod http;


fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut raw_request = String::new();

                // GPT 
                let mut buffer = [0; 1024];
                let mut request = Vec::new();

                loop {
                    let bytes_read = _stream.read(&mut buffer).expect("could not read bytes");

                    if bytes_read == 0 {
                        // Connection was closed by the client.
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

                // _stream.read_to_string(&mut raw_request).expect("could not read request");
                

                // parsing raw request into a struct
                let request = Request::try_from(raw_request).expect("could not parse request");

                // start building the response...
                // ...status line
                let status_line = if request.start_line.target == "/" {
                    StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"))
                } else {
                    StatusLine::new(String::from("HTTP/1.1"), 404, String::from("Not Found"))
                };

                // actual message struct
                let response_raw = HttpMessage::<StatusLine>::new(status_line, HashMap::new());

                // convert into raw response
                let response = Into::<String>::into(response_raw);
                println!("{:?}", response);

                _stream.write_all(response.as_bytes()).expect("could not send response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}