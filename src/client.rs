use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::fs::File;
use std::fs::OpenOptions;

use std::io::BufReader;

fn main() {
    let mut stream = TcpStream::connect("localhost:6142").unwrap();;
    println!("Successfully connected to server in port 6142");

    let mut val = [0 as u8; 1];
    loop {
        let mut data = [0 as u8; 4096];
        let read = stream.read(&mut data).unwrap();
        println!("{}", read);

        let mut file = OpenOptions::new().append(true).create(true).open("song.ogg").unwrap();
        if (read == 0) {
            file.flush();
            break;
        } else {
            file.write(&mut data);
            stream.write(&mut val);
        }
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("song.ogg").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    sink.sleep_until_end();

    println!("Terminated.");
} 
