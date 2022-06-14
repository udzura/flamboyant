use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::unix::prelude::RawFd;
use std::slice;
use std::sync::mpsc::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::spawn;

use libc::close;
use libc::FIONBIO;
use nix::errno::Errno;
use nix::sys::select;
use nix::sys::select::FdSet;
use nix::sys::socket::recv;
use nix::sys::socket::send;
use nix::sys::socket::MsgFlags;
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
    // let address = format!("127.0.0.1:{}", &port);

    // let listner = TcpListener::bind(&address).unwrap();
    // println!(
    //     "Listening: http://{}",
    //     listner.local_addr().unwrap().to_string()
    // );

    use nix::sys::socket::*;
    let sock = socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .unwrap();
    let on: i8 = 1;
    unsafe {
        if libc::ioctl(sock, FIONBIO, &on) < 0 {
            panic!("ioctl failed");
        }
    }

    let addr = SockaddrIn::new(127, 0, 0, 1, port.parse().unwrap());
    bind(sock, &addr).unwrap();
    listen(sock, 4096).unwrap();
    println!("Listening");

    let (sender, receiver) = sync_channel::<Vec<u8>>(4096);
    let (out_sender, out_receiver) = sync_channel::<Vec<u8>>(4096);

    let mut fdset = FdSet::new();
    fdset.insert(sock);

    let out_recvr = Arc::new(Mutex::new(out_receiver));
    let sendr = Arc::new(Mutex::new(sender));

    let th = spawn(move || loop {
        match select::select(None, Some(&mut fdset), None, None, None) {
            Ok(n) => {
                let sock = fdset.highest().unwrap();
                if n > 0 {
                    let go = sendr.clone();
                    let out = out_recvr.clone();
                    spawn(move || {
                        let fd: RawFd;
                        loop {
                            if let Ok(fd_) = accept(sock) {
                                fd = fd_;
                                break;
                            }
                        }
                        let data = process(fd);
                        go.lock().unwrap().send(data).unwrap();
                        let bytes = out.lock().unwrap().recv().unwrap();
                        //let bytes = out_receiver.recv().unwrap();
                        do_response(fd, &bytes);
                        unsafe { close(fd) };
                    });
                }
            }
            Err(e) => match e {
                Errno::EWOULDBLOCK => {}
                _ => {
                    panic!("{}", e)
                }
            },
        }
    });

    loop {
        let data = receiver.recv().unwrap();
        let req = String::from_utf8_lossy(&data);
        let reqline = req.lines().collect::<Vec<&str>>()[0];
        println!("{}", reqline);
        let response = handle_connection(app, &data);
        out_sender.send(response).unwrap();
    }

    th.join().unwrap();
}

//fn handle_connection(app: RubyValue, stream: &mut TcpStream) -> bool {
fn process(stream: RawFd) -> Vec<u8> {
    let mut buffer = [0; 4096];

    let mut read_bytes = 0;
    loop {
        match recv(stream, &mut buffer, MsgFlags::empty()) {
            Ok(n) => {
                if n > 0 {
                    read_bytes += n;
                } else {
                    if read_bytes > 0 {
                        break;
                    }
                }
            }
            Err(e) => match e {
                Errno::EWOULDBLOCK => {
                    if read_bytes > 0 {
                        break;
                    }
                }
                _ => {
                    panic!("{}", e)
                }
            },
        }
    }

    if read_bytes == 0 {
        return vec![0];
    }

    let index = buffer
        .iter()
        .enumerate()
        .find(|(_i, chr)| return **chr == ('\0' as u8))
        .unwrap();

    let data = (&buffer[..(index.0 + 1)]).to_vec();
    return data;
}

fn do_response(stream: RawFd, bytes: &[u8]) {
    send(stream, &bytes, MsgFlags::empty()).unwrap();
    //stream.write(&bytes).unwrap();
    //stream.flush().unwrap();

    return;
}

fn handle_connection(app: RubyValue, data: &[u8]) -> Vec<u8> {
    let request = CStr::from_bytes_with_nul(data).unwrap();
    let reqstring = unsafe { rb_utf8_str_new_cstr(request.as_ptr()) };

    let call = CString::new("call").unwrap();
    let args = vec![reqstring];
    let response = unsafe { rb_funcallv(app, rb_intern(call.as_ptr()), 1, args.as_ptr()) };
    let mut response = Box::new(response);

    // unsafe {
    //     rb_p(*response.as_mut());
    // }

    let bytes: *const i8 = unsafe { rb_string_value_ptr(response.as_mut()) };
    let len = unsafe { macros::RSTRING_LEN(response.as_ref().clone()) };

    let bytes_: &[i8] = unsafe { slice::from_raw_parts(bytes, len as usize) };
    let bytes: Vec<u8> = bytes_.iter().map(|v| *v as u8).collect();

    bytes
}
