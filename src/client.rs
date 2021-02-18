mod headers;
mod client_connection;

use client_connection::ClientConnection;
use std::net::{TcpStream};
use std::time::Duration;

fn main() {
    let mut client_id = String::new();
    std::io::stdin().read_line(&mut client_id).unwrap();
    let stream = TcpStream::connect("localhost:6142").unwrap();
    println!("Successfully connected to server in port 6142");
    stream.set_read_timeout(Some(Duration::from_millis(500)));

    let mut con = ClientConnection::new(client_id, stream);
    con.run()
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
