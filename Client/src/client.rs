use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    loop {
        let mut input = String::new();
        println!("Enter message: ");
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let _ = stream.write(input.as_bytes());
                let mut buf = [0; 512];
                let bytes_read = stream.read(&mut buf).unwrap();
                if bytes_read == 0 {
                    return;
                }
                println!("Received: {}", String::from_utf8_lossy(&buf[0..bytes_read]));
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
