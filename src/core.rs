use std::ffi::CString;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use rb_sys::*;

const _HELLO: &'static str = "<!DOCTYPE html>
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
pub unsafe extern "C" fn rb_flamboyant_serve(callback: RubyValue) -> RubyValue {
    serve(callback);
    return crate::ruby_ext::Nil.into();
}

fn serve(app: RubyValue) {
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
        handle_connection(app, stream);
    }
}

fn handle_connection(app: RubyValue, mut stream: TcpStream) {
    let mut buffer = [0; 4096];

    stream.read(&mut buffer).unwrap();

    let string = CString::new(&buffer[..]).unwrap();
    let reqstring = unsafe { rb_utf8_str_new_cstr(string.as_ptr()) };
    let call = CString::new("call").unwrap();
    let response = unsafe { rb_funcall(app, rb_intern(call.as_ptr()), 1, reqstring) };
    let mut response = Box::new(response);
    let bytes: *mut i8 = unsafe { rb_string_value_cstr(response.as_mut()) };
    let bytes = unsafe { CString::from_raw(bytes) };

    stream.write(bytes.as_bytes()).unwrap();
    stream.flush().unwrap();
}
