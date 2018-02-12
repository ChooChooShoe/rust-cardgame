
use std::net::{TcpStream,TcpListener};
use std::io::{Read,Write,Error};
use std::thread;

pub struct Client{}

pub fn start() -> thread::JoinHandle<()>
{
     thread::spawn(|| {
        //
    })
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