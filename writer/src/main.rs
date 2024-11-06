use std::time::Duration;
use std::{path::Path, thread};
use std::fs;
use rocksdb::{DBWithThreadMode, MultiThreaded, DB};

pub fn clear_db() {
    let db_path = Path::new("../database");

    if db_path.exists() {
        fs::remove_dir_all(db_path).unwrap();
    }
    fs::create_dir_all(db_path).unwrap();
}

fn main() {

    clear_db();
    let path = "../database";
    let db =
                             DBWithThreadMode::<MultiThreaded>::open_default(path).unwrap();
    
    let mut counter = 0;

    // for i in 0..100 {
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

    // for iter in db.iterator(rocksdb::IteratorMode::Start) {
    //     match iter {
    //         Ok((key,val)) => println!("{} -> {}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(val.to_vec()).unwrap()),
    //         Err(_) => return
    //     }
    // }

}
