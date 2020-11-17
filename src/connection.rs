use crate::headers;

use std::vec::Vec;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::Read;
use std::thread;
use headers::{Header, get_header_from_stream};

enum State {
    Connecting,
    Connected,
    Closed
}

pub struct Connection {
    client_id: String,
    socket :TcpStream,
    state: State
}

impl<'doom> Connection {
    pub fn init_connection(mut socket: &mut TcpStream) -> Arc<Mutex<&'static Connection>> {
        let header = get_header_from_stream(socket);
        if header.action != headers::CLIENT_ACK {
            println!("Got header but it's not an ack? Let's see what happens. action: {} length: {}",
                     header.action, header.length);
        } else {
            println!("Got header. action: {} length: {}", header.action, header.length);
        }
        let mut client_id_bytes = Vec::new();
        client_id_bytes.resize(header.length, 0);
        socket.read(&mut client_id_bytes);
        let mut ack_header = Header{action: 1, length:0};
        ack_header.send(&mut socket).unwrap(); // :J
        let mut con = Connection{client_id: String::from_utf8(client_id_bytes).unwrap(),
                                socket: socket.try_clone().unwrap(),
                                state: State::Connecting};
        let con_mutex = Arc::new(Mutex::new(&con));
        Connection::start_heartbeating(con_mutex);
        con_mutex
    }

    pub fn start_heartbeating(con_mutex: Arc<Mutex<&'static Connection>>) {
        let con_ref = Arc::clone(&con_mutex);
        // TODO: check connecting status
        thread::spawn(move || {
            let mut ack_header = Header{action: 1, length:0};
            loop {
                let mut con = con_ref.lock().unwrap();
                ack_header.send(&mut con.socket).unwrap();
                let mut header = get_header_from_stream(&mut con.socket);
                println!("Got data from client: {}, length: {}", con.client_id, header.length);
                let mut data_buf = Vec::new();
                data_buf.resize(header.length, 0);
                con.socket.read(&mut data_buf);
                drop(con);
                let echo = String::from_utf8(data_buf).unwrap();
                println!("{}", echo);
            }
        });
    }
}
