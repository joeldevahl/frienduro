extern crate frienduro;
use self::frienduro::{establish_connection, empty_db};

fn main() {
    let db = establish_connection();

    empty_db(&db).unwrap();
}
