use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use rb_sys::*;

const HELLO: &'static str = "<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\">
    <title>Hello!</title>
  </head>
  <body>
    <h1>Hello!</h1>
    <p>Hi from Rust</p>
  </body>
    </html>";

#[no_mangle]
pub unsafe extern "C" fn rb_flamboyant_serve(_slf: RubyValue) -> RubyValue {
    do_serve();
    return crate::ruby_ext::Nil.into();
}

fn do_serve() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }

    let listner = TcpListener::bind("127.0.0.1:17878").unwrap();
    println!(
        "Listening: http://{}",
        listner.local_addr().unwrap().to_string()
    );

    for stream in listner.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let response = vec![
        "HTTP/1.1 200 OK".to_string(),
        format!("Content-Length: {}", HELLO.len()),
        "".to_string(),
        HELLO.to_string(),
    ]
    .join("\r\n");

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
