use std::net::TcpStream;
use std::{thread, time};

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::con::{Connection, ConnectionSend};

fn main() {
    let stream = TcpStream::connect("localhost:6142").unwrap();
    let mut con = Connection::new(stream);

    let header = Header::new(headers::CLIENT_GET_LIBRARY, "abce".to_string());
    header.send(&mut con).unwrap();

    thread::sleep(time::Duration::from_millis(1000));
    let stream = TcpStream::connect("localhost:6142").unwrap();
    let mut con = Connection::new(stream);

    let abc = Header::new(headers::CLIENT_GET_QUEUE_INFO, "abce".to_string());
    abc.send(&mut con).unwrap();
}
