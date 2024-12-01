
use std::{hash::DefaultHasher, time::Instant};

use my_consistent_hashing::consistent_hashing::ConsistentHashing;


fn main() {

    let mut cons = ConsistentHashing::<DefaultHasher>::new(2);

    let begin = Instant::now();
    for i in 0..10 {
        cons.add_node(&format!("node{}", i)).unwrap();
    }
    println!("Done in {:?}", begin.elapsed());
    let x = cons.get_node(&"gay".to_string()).unwrap();
    println!("{}", x);

}