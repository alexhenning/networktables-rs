extern crate networktables;

// use networktables;
use networktables as nt;
use networktables::{Set, Get};

use std::io::Timer;
use std::time::Duration;


fn main() {
    println!("Starting");
    let client = match nt::Client::new("localhost:1735") {
        Ok(c) => c,
        Err(err) => panic!(format!("{}", err.kind))
    };

    client.set("/Counter".to_string(), 0f64);

    let mut timer = Timer::new().unwrap();
    let periodic = timer.periodic(Duration::milliseconds(1000));
    println!("Started");
    loop {
        // Print each of the types
        let b: Option<bool> = client.get("//Test".to_string());
        let n: Option<f64> = client.get("/Double".to_string());
        let s: Option<String> = client.get("/String".to_string());
        println!("{} {} {}", b, n, s);

        // Counter code
        let i: Option<f64> = client.get("/Counter".to_string());
        match i {
            Some(n) => {client.set("/Counter".to_string(), n+1f64);},
            None => (),
        };

        // Rate limit to once a second
        periodic.recv();
    }
    // println!("{} {} {}", client.Get::<bool>);
}
