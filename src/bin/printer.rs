extern crate networktables;

// use networktables;
use networktables::Client;

fn main() {
    println!("Starting");
    let client = match networktables::Client::new("localhost:1735") {
        Ok(c) => c,
        Err(err) => panic!(format!("{}", err.kind))
    };
    println!("Done");
    // println!("{} {} {}", client.Get::<bool>);
}
