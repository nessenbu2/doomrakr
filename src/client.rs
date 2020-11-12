use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    match TcpStream::connect("localhost:6142") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 6142");

            let mut data = [0 as u8; 100]; // using 6 byte buffer
            match stream.read(&mut data) {
                Ok(_) => {
                    let text = from_utf8(&data).unwrap();
                    println!("Got some shiz: {}", text);
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
} 
