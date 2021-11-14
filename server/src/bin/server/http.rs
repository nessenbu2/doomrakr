pub mod doom_http {
use std::sync::{Arc, Mutex};
use crate::Doomrakr;
use warp::Filter;

    pub fn run(doom: &Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        let runtime = tokio::runtime::Handle::try_current().unwrap();
        std::thread::spawn(move || runtime.block_on({
            let routes = warp::get()
                .map(move || {
                    println!("http got request");
                    let doom = doom_ref.lock().unwrap();
                    doom.get_all_status()
                });
            warp::serve(routes).run(([0, 0, 0, 0], 3030))
        }));
    }
}
