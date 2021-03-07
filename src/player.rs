use std::io::{BufReader, ErrorKind};
use std::collections::LinkedList;
use std::path::Path;

use crate::fs_walker;
use crate::fs_walker::Song;

pub struct Player {
    queue: LinkedList<Song>,
    sink: rodio::Sink
}

pub const base_path: &str = "/tmp/doomrakr";

impl Player {
    pub fn new() -> Player {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        match std::fs::create_dir(Path::new(base_path)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        Player {
            queue: LinkedList::new(),
            sink: sink
        }
    }

    pub fn play(&mut self, song: &Song) {
        let path = format!("{}/{}", base_path, fs_walker::Song::get_path(song));
        let song_file = std::fs::File::open(path).unwrap();
        self.sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());
        self.sink.play();
    }

    pub fn is_song_cached() {
    }

    pub fn init_stream() {
    }

    pub fn add_chunk() {
    }

    pub fn complete_tream() {
    }
}

