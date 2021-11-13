pub mod doom_http {
use std::sync::{Arc, Mutex};
use crate::Doomrakr;
use warp::Filter;

    pub fn run(doom: &Arc<Mutex<Doomrakr>>) {
//        let routes = warp::get()
//            .map(|| {
//                "abcd123"
//            });
//        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
        let doom_ref = doom.clone();

        let runtime = tokio::runtime::Handle::try_current().unwrap();
        std::thread::spawn(move || runtime.block_on({
            println!("big f ur momma");
            let doom_ref_ref = doom_ref.clone();
            let routes = warp::get()
                .map(move || {
                    let doom = doom_ref_ref.lock().unwrap();
                    println!("LOLOLOLOLOLO");
                    doom.get_all_status()
                });
            warp::serve(routes).run(([0, 0, 0, 0], 3030))
        }));
    }

    async fn inner_run(doom: &Arc<Mutex<Doomrakr>>) {
    }
}
