use crate::headers;

use std::vec::Vec;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::Read;
use std::ops::DerefMut;
use std::{thread, time};
use headers::{Header, get_header_from_stream};

enum State {
    Idle,
    Streaming,
    Closed
}

pub struct Connection {
    pub client_id: String,
    socket :TcpStream,
    state: State,
    is_changed: bool
}

fn con_main(con_ref: Arc<Mutex<Connection>>) {
    loop {
        let mut con = con_ref.lock().unwrap();

        if con.is_changed {
            // TODO
        }

        match con.state {
            State::Idle => heartbeat(con.deref_mut()),
            State::Streaming => send_chunk(con.deref_mut()),
            State::Closed => {
                clean_up(con.deref_mut());
                break;
            },
        }

        match con.state {
            State::Idle => {
                drop(con);
                thread::sleep(time::Duration::from_millis(10));
            },
            _ => ()
        }
    }
}

fn heartbeat(con: &mut Connection) {
    let mut peek = [0 as u8; 1];
    let len = con.socket.peek(&mut peek).expect("peek failed");

    if len != 0 {
        let header = get_header_from_stream(&mut con.socket);
        let mut data_buf = Vec::new();
        data_buf.resize(header.length, 0);
        con.socket.read(&mut data_buf).expect("Was given a lenght of data to read but failed");
        let echo = String::from_utf8(data_buf).unwrap();
        println!("Got heartbeat. echo: {}", echo);
    }

    let mut ack_header = Header{action:headers::SERVER_ACK, length:0};
    ack_header.send(&mut con.socket).unwrap();
}

fn send_chunk(con: &mut Connection) {
}

fn clean_up(con: &mut Connection) {
}

fn start_heartbeating(con_mutex: Arc<Mutex<Connection>>) {
    thread::spawn(move || {
        con_main(Arc::clone(&con_mutex))
    });
}

impl Connection {

    pub fn init_connection(mut socket: &mut TcpStream) -> Arc<Mutex<Connection>> {
        let header = get_header_from_stream(socket);
        if header.action != headers::CLIENT_HELLO {
            println!("Got header but it's not a hello? Let's see what happens. action: {} length: {}",
                     header.action, header.length);
        } else {
            println!("Got header. action: {} length: {}", header.action, header.length);
        }
        let mut client_id_bytes = Vec::new();
        client_id_bytes.resize(header.length, 0);

        // TODO: this expect will crash the server
        socket.read(&mut client_id_bytes).expect("Failed to read bytes");
        let mut ack_header = Header{action: 1, length:0};
        ack_header.send(&mut socket).unwrap(); // :J
        let con_mutex = Arc::new(Mutex::new(Connection{
                                client_id: String::from_utf8(client_id_bytes).unwrap(),
                                socket: socket.try_clone().unwrap(),
                                state: State::Idle,
                                is_changed: false}));
        start_heartbeating(con_mutex.clone());
        con_mutex
    }

    pub fn send_song(&self, song_path: String) {

    }

}
