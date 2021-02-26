use std::io::BufReader;
use std::collections::LinkedList;

use crate::fs_walker;
use crate::fs_walker::Song;

pub struct Player {
    queue: LinkedList<Song>,
    sink: rodio::Sink
}

impl Player {
    pub fn new() -> Player {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        Player {
            queue: LinkedList::new(),
            sink: sink
        }
    }

    pub fn play(&mut self, song: &Song) {
        let path = format!("./{}", fs_walker::Song::get_path(song));
        let song_file = std::fs::File::open("examples/music.ogg").unwrap();
        self.sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());
    }
}

