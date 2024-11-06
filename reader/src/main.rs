
use std::{env, thread, time::Duration};

use rocksdb::{DBWithThreadMode, MultiThreaded};

fn main() {
    
    let args: Vec<String> = env::args().collect();

    println!("Starting ...");
    println!("Git test ...");

    let opts = rocksdb::Options::default();
    let db =
            DBWithThreadMode::<MultiThreaded>::open_as_secondary(&opts,
                &args[1],
                &args[2]).unwrap();
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