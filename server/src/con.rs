use std::net::TcpStream;
use std::io::{Read,Write};

pub struct Connection {
    stream: TcpStream
}

// Traits to send and get an object over a connection. If ConnectionSend
// is implemented for an object, ConnectionGet should be implemented as well.
//
// I'm not sure if something like that is possible in rust (cursory google search
// doesn't show anything) so I'm not going to try. Maybe if it's possible I can
// add it in the future. :3
//
// Returns the number of bytes written or an error message
pub trait ConnectionSend {
    fn send(&self, con: &mut Connection) -> Result<usize, String>;
}

// Returns the number of bytes read or an error message
pub trait ConnectionGet {
    fn get(con: &mut Connection) -> Result<Self, String> where Self: Sized;
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: stream 
        }
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize, String> {
        match self.stream.write(buf) {
            Ok(bytes) => Ok(bytes),
            Err(error) => Err(error.to_string())
        }
    }

    pub fn get(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        match self.stream.read(buf) {
            Ok(bytes) => Ok(bytes),
            Err(error) => Err(error.to_string())
        }
    }

    pub fn get_exact(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        match self.stream.read_exact(buf) {
            Ok(_) => Ok(0),
            Err(error) => Err(error.to_string())
        }
    }

    pub fn has_data(&self) -> bool {
        let mut peek = [0 as u8; 1];
        match self.stream.peek(&mut peek) {
            Ok(_) => true,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => false,
            Err(e) => panic!("hit some IO error {}", e)
        }
    }
}
