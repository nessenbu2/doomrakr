extern crate chrono;

pub mod logger {

    use ::chrono::prelude::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    pub fn log(message: &str) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(gen_filename())
            .unwrap();

        file.write_all(message.as_bytes()).expect("Unable to write data");
    }

    fn gen_filename() -> String {
        let local_time = Local::now();
        format!("/tmp/doomrakr.{month}.{day}.{hour}.log",
                month = local_time.month(),
                day = local_time.day(),
                hour = local_time.hour())
    }
}
