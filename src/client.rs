use std::net::{TcpStream};
use std::io::{self, Read, Write};
use std::str::from_utf8;
use std::fs::File;
use std::fs::OpenOptions;

use std::io::BufReader;

fn main() {
    let mut client_id = String::new();
    std::io::stdin().read_line(&mut client_id).unwrap();
    let mut stream = TcpStream::connect("localhost:6142").unwrap();;
    println!("Successfully connected to server in port 6142");

    let mut ack_header = Header{action:1, length:client_id.len()};
    ack_header.send(&mut stream).unwrap();
    stream.write(client_id.as_bytes());
    loop {
        let got_msg_header = get_header_from_stream(&mut stream);
        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();
        let mut msg_header = Header{action:123, length:msg.len()};
        msg_header.send(&mut stream);
        stream.write(msg.as_bytes());
        /*
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
        */
    }

    /*
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("song.ogg").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    sink.sleep_until_end();

    println!("Terminated.");
    */
} 

// BIG TODO: move all this shiz to a helper file

// ACTIONS
const CLIENT_ACK: u8 = 0;
const SERVER_ACK: u8 = 1;
const STRING_DATA: u8 = 123;

pub struct Header {
    action: u8,
    length: usize
}

impl Header {
    fn send(&mut self, stream: &mut TcpStream) -> io::Result<(usize)> {
        stream.write(&u8::to_be_bytes(self.action))
              .and_then(|_| stream.write(&usize::to_be_bytes(self.length)))
    }
}

// It's assumed that buf is empty. It only appends, so guess if you want that
// it'll work, but you're a braver man than I

pub fn get_header_from_stream(stream: &mut TcpStream) -> Header {
    let mut action = [0 as u8; 1];
    let mut length = [0 as u8; 8];
    stream.read(&mut action).unwrap();
    stream.read(&mut length).unwrap();
    Header {action: u8::from_be_bytes(action), length: usize::from_be_bytes(length)}
}
