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
    fn get(&self, con: &mut Connection) -> Result<Self, String> where Self: Sized;
}

impl Connection {
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

    pub fn has_data(&self) -> bool {
        false
    }
}
