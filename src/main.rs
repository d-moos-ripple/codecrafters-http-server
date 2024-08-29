use std::{collections::HashMap, io::{Read, Write}, net::TcpListener};
use http::{message::HttpMessage, request::Request, response::{Response, StatusLine}};

mod http;


fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut raw_request = String::new();
                _stream.read_to_string(&mut raw_request).expect("could not read request");
                let request = Request::try_from(raw_request).expect("could not parse request");

                let status_line = if request.start_line.target == "/" {
                    StatusLine::new(String::from("HTTP/1.1"), 200, String::from("OK"))
                } else {
                    StatusLine::new(String::from("HTTP/1.1"), 404, String::from("Not Found"))
                };

                let response = HttpMessage::<StatusLine>::new(status_line, HashMap::new());

                _stream.write_all(Into::<String>::into(response).as_bytes()).expect("could not send response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}