pub mod doom_http {
use std::sync::{Arc, Mutex};
use crate::Doomrakr;
use warp::Filter;

    pub fn run(doom: &Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        let runtime = tokio::runtime::Handle::try_current().unwrap();
        std::thread::spawn(move || runtime.block_on({

            let doom = doom_ref.clone();
            let base = warp::path("status")
                .map(move || {
                    println!("http got request");
                    doom.lock().unwrap().get_all_status()
                });

            // path here is: GET /play/ARTIST/ALBUM/SONG
            let doom = doom_ref.clone();
            let play = warp::path!("play" / String / String / String / String)
                .map(move |client_id, artist, album, song|{
                    doom.lock().unwrap().add_song(client_id, artist, album, song);
                    "artist"
                });

            let doom = doom_ref.clone();
            let pause = warp::path!("pause" / String)
                .map(move |client_id|{
                    doom.lock().unwrap().pause_song(client_id);
                    "paused"
                });
            // GET /status
            // GET /play/:string/:string/:string
            let routes = base.or(play)
                .or(pause);
            warp::serve(routes).run(([0, 0, 0, 0], 3030))
        }));
    }
}
