use std::{env, thread, time::Duration};
use rocksdb::{DBWithThreadMode, IteratorMode, MultiThreaded};
use rocket::{get, launch, routes, State};
use std::sync::Mutex;


type DbInstance = DBWithThreadMode<MultiThreaded>;

#[macro_use]
extern crate rocket;

struct Database {
    db: Mutex<DbInstance>,
}

#[get("/read")]
fn read_data(db: &State<Database>) -> String {
    let db = db.db.lock().unwrap();
    
    // Try to catch up with the primary database
    if let Err(e) = db.try_catch_up_with_primary() {
        return format!("Error syncing with primary: {}", e);
    }

    // Read the database content and concatenate into a single string
    let mut result = String::new();
    for iter in db.iterator(IteratorMode::Start) {
        match iter {
            Ok((key, val)) => {
                let key_str = String::from_utf8(key.to_vec()).unwrap_or_default();
                let val_str = String::from_utf8(val.to_vec()).unwrap_or_default();
                result.push_str(&format!("{} -> {}\n", key_str, val_str));
            }
            Err(_) => return "Error reading from database".to_string(),
        }
    }
    result
}

#[launch]
fn rocket() -> _ {
    // Retrieve environment variables
    let db_path = env::var("DATABASE_PATH").expect("DATABASE_PATH not set");
    let secondary_path = env::var("SECONDARY_PATH").expect("SECONDARY_PATH not set");

    println!("Starting Rocket server...");

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

    let db_instance = Database {
        db: Mutex::new(db),
    };

    rocket::build()
        .manage(db_instance)
        .mount("/", routes![read_data])
}


// use std::{env, thread, time::Duration};

// use rocksdb::{DBWithThreadMode, MultiThreaded};

// fn main() {
    

//     let db_path = env::var("DATABASE_PATH").unwrap();
//     let secondary_path = env::var("SECONDARY_PATH").unwrap();
    
//     println!("Starting ...");
//     println!("Git test ...");

//     let opts = rocksdb::Options::default();
//     let db = loop {
//         match DBWithThreadMode::<MultiThreaded>::open_as_secondary(&opts, &db_path, &secondary_path) {
//             Ok(database) => break database,
//             Err(e) => {
//                 println!("Failed to open DB: {}", e);
//                 thread::sleep(Duration::from_secs(1));
//             }
//         }
//     };
    
//     loop {
//         db.try_catch_up_with_primary().unwrap();
//         for iter in db.iterator(rocksdb::IteratorMode::Start) {
//             match iter {
//                 Ok((key,val)) => println!("{} -> {}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(val.to_vec()).unwrap()),
//                 Err(_) => return
//             }
//         }
//         println!("----------------------");
//         thread::sleep(Duration::from_secs(5));
//     }

// }