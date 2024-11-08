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

    let args: Vec<String> = env::args().collect();

    // let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
    // let config = aws_config::defaults(BehaviorVersion::latest())
    //     .region(region_provider)
    //     .load()
    //     .await;

    // let client = Client::new(&config);
    // let region = config.region().unwrap();
    // let constraint = aws_sdk_s3::types::BucketLocationConstraint::from(region.as_ref());
    // let _cfg = aws_sdk_s3::types::CreateBucketConfiguration::builder()
    //     .location_constraint(constraint.clone())
    //     .build();


    clear_db(&args[1]);
    let path = &args[1];
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
            // upload_object(&client, "sdk-test-bucket-v69",
            //                         format!("{}/LOG", &args[1]).as_str(),
            //                         format!("log_{}", counter).as_str()).await.unwrap();
            thread::sleep(Duration::from_millis(5_000));
        }
    }

}
