use std::io::{BufReader, ErrorKind, Write};
use std::collections::LinkedList;
use std::path::Path;
use std::fs::OpenOptions;

use doomrakr::song::Song;

const BASE_PATH: &str = "/tmp/doomrakr";
const STREAMING_PATH: &str = "/tmp/doomrakr/streaming";

fn get_path_for_song(song: &Song) -> String {
    format!("{}/{}", BASE_PATH, Song::get_path(song))
}

fn get_for_stream(song: &Song) -> String {
    format!("{}/{}", STREAMING_PATH, Song::get_path(song))
}

fn get_dir_for_song(song: &Song) -> String {
    format!("{}/{}/{}", BASE_PATH, song.artist, song.album)
}

fn get_dir_for_stream(song: &Song) -> String {
    format!("{}/{}/{}", STREAMING_PATH, song.artist, song.album)
}

pub struct Player {
    // Songs not yet fed to the sink
    queued_songs: LinkedList<Song>,
    // Songs already fed to the sink. This shouldn't be more than 2
    playing_songs: LinkedList<Song>,
    // Marked witn '_' since we need to keep the stream in scope
    // or rodio won't be able to play. Currently, this isn't used
    _stream: rodio::OutputStream,
    sink: rodio::Sink
}

impl Player {
    pub fn new() -> Player {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();

        // TODO: probably check if this is a file or not?
        //       also add a helper
        match std::fs::create_dir(Path::new(BASE_PATH)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        // Don't mess this up :)
        match std::fs::remove_dir_all(STREAMING_PATH) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("Can't purge songs that failed to stream: {}", e)
                }
            }
        };

        match std::fs::create_dir(Path::new(STREAMING_PATH)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }

        Player {
            queued_songs: LinkedList::new(),
            playing_songs: LinkedList::new(),
            _stream: stream,
            sink: rodio::Sink::try_new(&handle).unwrap()
        }
    }

    pub fn add_to_queue(&mut self, song: Song) {
        self.queued_songs.push_back(song);
    }

    pub fn pause(&mut self) {
        self.sink.pause()
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn get_queue(&self) -> LinkedList<Song> {
        let mut ret = self.playing_songs.clone();
        ret.append(&mut self.queued_songs.clone());
        ret
    }

    pub fn resume(&mut self) {
        self.sink.play()
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
        match std::fs::remove_file(path) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        };

        match std::fs::create_dir_all(get_dir_for_stream(song)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
    }

    pub fn add_chunk(song: &Song, data: &mut [u8; 4096]) {
        let path = get_for_stream(song);
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap()
            .write(data) {
                Ok(_) => (),
                Err(err) => print!("{}", err.to_string())
            };
    }

    pub fn complete_stream(song: &Song) {
        match std::fs::create_dir_all(get_dir_for_song(song)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        let path = get_for_stream(song);
        let mv_path = get_path_for_song(song);
        match std::fs::rename(path, mv_path) {
            Ok(_) => (),
            Err(err) => println!("{}", err.to_string())
        }
    }

    pub fn maybe_enquque_song(&mut self) {
        // First make sure playing_songs reflects the sink length
        while self.sink.len() < self.playing_songs.len() {
            self.playing_songs.pop_front();
        }

        if self.sink.len() > self.playing_songs.len() {
            println!("sink len is greater than our queue len somehow. sink len: {}, queue len: {}",
                     self.sink.len(), self.playing_songs.len());
        }

        if self.sink.len() < 2 {
            match self.queued_songs.pop_front() {
                None => (),
                Some(song) => {
                    let path = get_path_for_song(&song);
                    self.playing_songs.push_back(song);

                    // TODO: handle cases where this doesn't exists
                    let song_file = std::fs::File::open(path).unwrap();
                    self.sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());
                }
            }
        }
    }
}

