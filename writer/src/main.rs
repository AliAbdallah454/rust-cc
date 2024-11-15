use rocket::{post, routes, serde::json::Json, State};
use rocksdb::{DBWithThreadMode, MultiThreaded};
use serde::Deserialize;
use std::sync::Mutex;
use std::{env, path::Path, fs, thread, time::Duration};

type DbInstance = DBWithThreadMode<MultiThreaded>;

#[macro_use]
extern crate rocket;

struct Database {
    db: Mutex<DbInstance>,
}

#[derive(Deserialize)]
struct WriteRequest {
    key: String,
    value: String,
}

#[post("/write", data = "<write_req>")]
fn write_data(db: &State<Database>, write_req: Json<WriteRequest>) -> &'static str {
    let db = db.db.lock().unwrap();
    
    // Write the key-value pair to the database
    if let Err(e) = db.put(&write_req.key, &write_req.value) {
        eprintln!("Error writing to DB: {}", e);
        return "Failed to write data";
    }

    "Data written successfully"
}

fn clear_db(path: &str) {
    let db_path = Path::new(path);
    if db_path.exists() {
        fs::remove_dir_all(db_path).unwrap();
    }
    fs::create_dir_all(db_path).unwrap();
}

#[launch]
fn rocket() -> _ {
    let db_path = env::var("DATABASE_PATH").expect("DATABASE_PATH not set");
    // clear_db(&db_path);

    println!("Starting Rocket server...");

    // Open RocksDB database
    let db = DBWithThreadMode::<MultiThreaded>::open_default(&db_path)
        .expect("Failed to open database");

    let db_instance = Database {
        db: Mutex::new(db),
    };

    rocket::build()
        .manage(db_instance)
        .mount("/", routes![write_data])
}


// use std::time::Duration;
// use std::{path::Path, thread};
// use std::{env, fs};
// use rocksdb::{DBWithThreadMode, MultiThreaded};

// pub fn clear_db(path: &str) {
//     let db_path = Path::new(path);

//     if db_path.exists() {
//         fs::remove_dir_all(db_path).unwrap();
//     }
//     fs::create_dir_all(db_path).unwrap();
// }

// fn main() {

//     let path = env::var("DATABASE_PATH").unwrap();
//     let db =
//                              DBWithThreadMode::<MultiThreaded>::open_default(path).unwrap();
    
//     let mut counter = 0;

//     loop {
//         counter += 1;
//         let key = format!("key_{}", counter);
//         let val = format!("val_{}", counter);
//         println!("Putting :)");
//         db.put(key, val).unwrap();
//         thread::sleep(Duration::from_millis(1_000));
//         if counter % 10 == 0 {
//             println!("Flushing ...");
//             db.flush().unwrap();
//             thread::sleep(Duration::from_millis(5_000));
//         }
//     }

// }
