
use std::net::{TcpStream,TcpListener};
use std::io::{Read,Write,Error};
use std::thread;
use std::thread::JoinHandle;

pub struct Server
{
    listener_handle: JoinHandle<()>,
}

impl Server {
    pub fn new() -> Server
    {
        let h = thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
            info!("Listening for client connections on port {}", 8080);
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(|| {
                            on_client_connect(stream)
                        });
                    }
                    Err(e) => {
                        error!("Unable to connect to client: {}", e);
                    }
                }
            }
        });

        Server{ listener_handle: h}
    }
}

fn on_client_connect(stream: TcpStream) {
    println!("New Connection");
    on_read(&stream);
    on_write(&stream);
    println!("Done Connection");
}

fn on_read(mut stream: &TcpStream) {
    let mut buf = [0x20; 2024];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            println!("{}", req_str);
        },
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

fn on_write(mut stream: &TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";
    match stream.write(response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}