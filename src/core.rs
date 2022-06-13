use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::net::TcpStream;
use std::slice;

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
pub unsafe extern "C" fn rb_flamboyant_serve(_slf: RubyValue, callback: RubyValue) -> RubyValue {
    serve(callback);
    return crate::ruby_ext::Nil.into();
}

fn serve(app: RubyValue) {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }

    let port = env::var("PORT").unwrap();
    let address = format!("127.0.0.1:{}", &port);

    let listner = TcpListener::bind(&address).unwrap();
    println!(
        "Listening: http://{}",
        listner.local_addr().unwrap().to_string()
    );

    listner.set_nonblocking(true).unwrap();

    loop {
        match listner.accept() {
            Ok((mut stream, _addr)) => while !handle_connection(app, &mut stream) {},
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {}
                _ => {
                    panic!("{}", e)
                }
            },
        }
    }
}

fn handle_connection(app: RubyValue, stream: &mut TcpStream) -> bool {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::WouldBlock => return false,
            _ => {
                panic!("{}", e)
            }
        },
    }

    let index = buffer
        .iter()
        .enumerate()
        .find(|(_i, chr)| return **chr == ('\0' as u8))
        .unwrap();
    let request = CStr::from_bytes_with_nul(&buffer[..(index.0 + 1)]).unwrap();
    eprintln!("read: {:?}", &request);

    let mut dest: Vec<i8> = Vec::with_capacity(index.0 + 2);
    let ptr = dest.into_boxed_slice().as_mut_ptr();

    eprintln!("copying");
    unsafe {
        std::ptr::copy_nonoverlapping(request.as_ptr(), ptr, index.0 + 2);
    }
    eprintln!("copyed");

    let dest = unsafe { CStr::from_ptr(ptr) };

    let reqstring = unsafe { rb_utf8_str_new_cstr(dest.as_ptr()) };

    unsafe { rb_p(app) };
    let call = CString::new("call").unwrap();
    let args = vec![reqstring];
    let response = unsafe { rb_funcallv(app, rb_intern(call.as_ptr()), 1, args.as_ptr()) };
    let mut response = Box::new(response);

    let bytes: *const i8 = unsafe { rb_string_value_ptr(response.as_mut()) };
    let len = unsafe { macros::RSTRING_LEN(response.as_ref().clone()) };

    let bytes_: &[i8] = unsafe { slice::from_raw_parts(bytes, len as usize) };
    let bytes: Vec<u8> = bytes_.iter().map(|v| *v as u8).collect();

    stream.write(&bytes).unwrap();
    stream.flush().unwrap();

    return true;
}
