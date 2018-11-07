extern crate frienduro;
use self::frienduro::{establish_connection, create_db};

fn main() {
    let db = establish_connection();
    match create_db(&db) {
        Ok(_) => (),
        Err(err) => println!("Failed to create DB: {}", err),
    }
}
