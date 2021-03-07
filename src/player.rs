use std::io::{BufReader, ErrorKind, Write};
use std::collections::LinkedList;
use std::path::Path;
use std::fs::File;
use std::fs::OpenOptions;

use crate::fs_walker;
use crate::fs_walker::Song;

pub struct Player {
    queue: LinkedList<Song>,
    sink: rodio::Sink
}

const base_path: &str = "/tmp/doomrakr";
const streaming_path: &str = "/tmp/doomrakr/streaming";

fn get_path_for_song(song: &Song) -> String {
    format!("{}/{}", base_path, fs_walker::Song::get_path(song))
}

fn get_for_stream(song: &Song) -> String {
    format!("{}/{}", streaming_path, fs_walker::Song::get_path(song))
}

impl Player {
    pub fn new() -> Player {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        // TODO: probably check if this is a file or not?
        //       also add a helper
        match std::fs::create_dir(Path::new(base_path)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        // Don't mess this up :)
        std::fs::remove_dir_all(streaming_path);
        match std::fs::create_dir(Path::new(streaming_path)) {
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
        let path = get_path_for_song(song);
        // TODO: handle cases where this doesn't exists
        let song_file = std::fs::File::open(path).unwrap();
        self.sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());
        self.sink.play();
    }

    // Rethink these. Can probably do this waaaaay smarter
    pub fn is_song_cached(song: &Song) -> bool {
        let path = get_path_for_song(song);
        Path::new(&path).exists()
    }

    pub fn is_song_streaming(song: &Song) -> bool {
        let path = get_for_stream(song);
        Path::new(&path).exists()
    }

    pub fn init_stream(song: &Song) {
        let path = get_for_stream(song);
        // Just nuke the file for now, it'll be created when we stream chunks
        std::fs::remove_file(path);
    }

    pub fn add_chunk(song: &Song, data: &mut [u8; 4096]) {
        let path = get_for_stream(song);
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap()
            .write(data);
    }

    pub fn complete_stream(song: &Song) {
        let path = get_for_stream(song);
        let mv_path = get_path_for_song(song);
        std::fs::rename(path, mv_path);
    }
}

