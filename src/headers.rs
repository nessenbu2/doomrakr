use std::net::{TcpStream};
use std::io::{self, Read, Write};

use crate::con::{Connection, ConnectionSend, ConnectionGet};

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
pub const CLIENT_SONG_CACHED: u8 = 8;

pub struct Header {
    pub action: u8,
    pub id: String,
}

impl ConnectionSend for Header {
    fn send(&self, con: &mut Connection) -> Result<usize, String> {
        let mut written = match con.send(&u8::to_be_bytes(self.action)) {
            Ok(bytes) => bytes,
            Err(error) => return Err(error)
        };
        written + match con.send(&usize::to_be_bytes(self.id.len())) {
            Ok(bytes) => bytes,
            Err(error) => return Err(error)
        };
        written + match con.send(self.id.as_bytes()) {
            Ok(bytes) => bytes,
            Err(error) => return Err(error)
        };
        Ok(written)
    }
}

impl ConnectionGet for Header {
    fn get(con: &mut Connection) -> Result<Self, String> {
        let mut action = [0 as u8; 1];
        let mut length = [0 as u8; 8];
        // TODO check that the right amount of bytes were read
        con.get(&mut action)?;
        con.get(&mut length)?;
        let length = usize::from_be_bytes(length);
        let mut id_bytes = vec![0u8; length];
        con.get(&mut id_bytes)?;
        let id = match String::from_utf8(id_bytes) {
            Ok(id) => id,
            Err(message) => return Err(message.to_string())
        };

        Ok(Header::new(u8::from_be_bytes(action), id))
    }
}

impl Header {
    pub fn new(action: u8, id: String) -> Header {
        Header {
            action: action,
            id: id
        }
    }
}

