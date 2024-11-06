
use std::{thread, time::Duration};

use rocksdb::{DBWithThreadMode, MultiThreaded};

fn main() {
    
    println!("Starting ...");
    println!("Git test ...");

    let opts = rocksdb::Options::default();
    let db =
            DBWithThreadMode::<MultiThreaded>::open_as_secondary(&opts,
                "../database",
                "../database2").unwrap();
    loop {
        db.try_catch_up_with_primary().unwrap();
        for iter in db.iterator(rocksdb::IteratorMode::Start) {
            match iter {
                Ok((key,val)) => println!("{} -> {}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(val.to_vec()).unwrap()),
                Err(_) => return
            }
        }
        thread::sleep(Duration::from_secs(2));
    }

}