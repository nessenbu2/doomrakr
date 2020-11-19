use crate::headers;

use std::thread;
use std::time::SystemTime;
use std::net::TcpStream;
use std::io::{Read, Write};
use headers::{Header, get_header_from_stream};

enum ClientState {
    Idle,
    Streaming,
    Playing,
    Closed
}

pub struct ClientConnection {
    client_id: String,
    stream: TcpStream,
    state: ClientState,
    last_hb_time: SystemTime
    // may want a last_ack_time if i wanna be really robust
}

fn check_for_commands(con: &mut ClientConnection) {
    let mut peek = [0 as u8; 1];
    match con.stream.peek(&mut peek) {
        Ok(_) => {
            let got_msg_header = get_header_from_stream(&mut con.stream);
            println!("got header. action {}, len: {}", got_msg_header.action, got_msg_header.length);
            // TODO: update state based on action
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // NOOP
        }
        Err(e) => panic!("hit some IO error {}", e)
    }
}

fn maybe_heartbeat(con: &mut ClientConnection) {
    if SystemTime::now().duration_since(con.last_hb_time).unwrap().as_secs() >= 1 {
        let mut msg_header = Header{action:123, length: con.client_id.len()};
        msg_header.send(&mut con.stream);
        con.stream.write(con.client_id.as_bytes());
        println!("sent hb");
        con.last_hb_time = SystemTime::now();
    }
}

fn recv_chunk(con: &mut ClientConnection) {
}

fn check_stream(con: &mut ClientConnection) {
}

impl ClientConnection {
    pub fn run(&mut self) {
        let mut hello_header = Header{action:123, length: self.client_id.len()};
        hello_header.send(&mut self.stream).unwrap();
        self.stream.write(self.client_id.as_bytes());
        loop {
            check_for_commands(self);
            match self.state{
                ClientState::Idle => maybe_heartbeat(self),
                ClientState::Streaming => recv_chunk(self),
                ClientState::Playing => check_stream(self),
                ClientState::Closed => break
            }
        }
    }

    pub fn new(client_id: String, stream: TcpStream) -> ClientConnection {
        ClientConnection{
            client_id: client_id,
            stream: stream,
            state: ClientState::Idle,
            last_hb_time: SystemTime::now()
        }
    }
}
