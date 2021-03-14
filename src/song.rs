use crate::con::{Connection, ConnectionSend, ConnectionGet};

#[derive(Clone)]
pub struct Song {
    pub artist: String,
    pub album: String,
    pub name: String,
}

impl Song {
    pub fn new(artist: String, album: String, name: String) -> Song {
        Song {
            artist: artist,
            album: album,
            name: name
        }
    }

    pub fn get_path(song: &Song) -> String {
        format!("{}/{}/{}",song.artist.clone(), song.album.clone(), song.name.clone())
    }
}

impl ConnectionGet for Song {
    fn get(con: &mut Connection) -> Result<Self, String> where Self: Sized {
        let mut length = [0 as u8; 8];
        // Read lenghts of song names
        con.get(&mut length)?;
        let artist_length = usize::from_be_bytes(length);
        con.get(&mut length)?;
        let album_length = usize::from_be_bytes(length);
        con.get(&mut length)?;
        let song_length = usize::from_be_bytes(length);

        let mut artist_bytes = vec![0u8; artist_length];
        let mut album_bytes = vec![0u8; album_length];
        let mut song_bytes = vec![0u8; song_length];
        con.get(&mut artist_bytes)?;
        con.get(&mut album_bytes)?;
        con.get(&mut song_bytes)?;

        let artist = match String::from_utf8(artist_bytes) {
            Ok(artist) => artist,
            Err(message) => return Err(message.to_string())
        };

        let album = match String::from_utf8(album_bytes) {
            Ok(album) => album,
            Err(message) => return Err(message.to_string())
        };

        let song = match String::from_utf8(song_bytes) {
            Ok(song) => song,
            Err(message) => return Err(message.to_string())
        };

        Ok(Song::new(artist, album, song))
    }
}

impl ConnectionSend for Song {
    fn send(&self, con: &mut Connection) -> Result<usize, String> {
        let bytes = con.send(&usize::to_be_bytes(self.artist.len()))?;
        con.send(&usize::to_be_bytes(self.album.len()))?;
        con.send(&usize::to_be_bytes(self.name.len()))?;
        con.send(&mut self.artist.as_bytes())?;
        con.send(&mut self.album.as_bytes())?;
        Ok(bytes + con.send(&mut self.name.as_bytes())?)
    }
}
