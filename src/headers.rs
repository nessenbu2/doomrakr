use std::net::{TcpStream};
use std::io::{self, Read, Write};

// ACTIONS
pub const CLIENT_HELLO: u8 = 0;
pub const CLIENT_ACK: u8 = 1;
pub const CLIENT_HB: u8 = 2;
pub const SERVER_ACK: u8 = 3;
pub const SERVER_INIT_STREAM: u8 = 4;
pub const SERVER_STREAM_CHUNK: u8 = 5;
pub const SERVER_STREAM_FINISHED: u8 = 6;
pub const SERVER_START: u8 = 7;
pub const SERVER_PAUSE: u8 = 8;

pub struct Header {
    pub action: u8,
    pub length: usize
}

impl Header {
    pub fn send(&mut self, stream: &mut TcpStream) -> io::Result<usize> {
        stream.write(&u8::to_be_bytes(self.action))
              .and_then(|_| stream.write(&usize::to_be_bytes(self.length)))
    }
}

// It's assumed that buf is empty. It only appends, so guess if you want that
// it'll work, but you're a braver man than I
pub fn get_header_from_stream(stream: &mut TcpStream) -> Header {
    let mut action = [0 as u8; 1];
    let mut length = [0 as u8; 8];
    stream.read(&mut action).unwrap(); // TODO :)
    stream.read(&mut length).unwrap(); // TODO :)
    Header {action: u8::from_be_bytes(action), length: usize::from_be_bytes(length)}
}
