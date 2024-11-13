
use std::{env, thread, time::Duration};

use rocksdb::{DBWithThreadMode, MultiThreaded};

fn main() {
    

    let db_path = env::var("DATABASE_PATH").unwrap();
    let secondary_path = env::var("SECONDARY_PATH").unwrap();
    
    println!("Starting ...");
    println!("Git test ...");

    let opts = rocksdb::Options::default();
    let db = loop {
        match DBWithThreadMode::<MultiThreaded>::open_as_secondary(&opts, &db_path, &secondary_path) {
            Ok(database) => break database,
            Err(e) => {
                println!("Failed to open DB: {}", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    };
    
    loop {
        db.try_catch_up_with_primary().unwrap();
        for iter in db.iterator(rocksdb::IteratorMode::Start) {
            match iter {
                Ok((key,val)) => println!("{} -> {}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(val.to_vec()).unwrap()),
                Err(_) => return
            }
        }
        println!("----------------------");
        thread::sleep(Duration::from_secs(5));
    }

}