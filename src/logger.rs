extern crate chrono;

pub mod logger {

    use logger::chrono::prelude::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    pub fn log(message: String) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(gen_filename())
            .unwrap();

        file.write_all(message.as_bytes()).expect("Unable to write data");
    }

    fn gen_filename() -> String {
        let local_time = Local::now();
        format!("/tmp/doomrakr/doomrakr.{month}.{day}.{hour}.log",
                month = local_time.month(),
                day = local_time.day(),
                hour = local_time.hour())
    }
}
