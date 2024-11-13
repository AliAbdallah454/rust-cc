use std::time::Duration;
use std::{path::Path, thread};
use std::{env, fs};
use rocksdb::{DBWithThreadMode, MultiThreaded};

pub fn clear_db(path: &str) {
    let db_path = Path::new(path);

    if db_path.exists() {
        fs::remove_dir_all(db_path).unwrap();
    }
    fs::create_dir_all(db_path).unwrap();
}

fn main() {

    let path = env::var("DATABASE_PATH").unwrap();
    let db =
                             DBWithThreadMode::<MultiThreaded>::open_default(path).unwrap();
    
    let mut counter = 0;

    loop {
        counter += 1;
        let key = format!("key_{}", counter);
        let val = format!("val_{}", counter);
        println!("Putting :)");
        db.put(key, val).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        if counter % 10 == 0 {
            println!("Flushing ...");
            db.flush().unwrap();
            thread::sleep(Duration::from_millis(5_000));
        }
    }

}
