extern crate frienduro;
use self::frienduro::establish_connection;

const EMPTY_DB_SQL: &'static str = include_str!("empty_db.sql");
const CREATE_DB_SQL: &'static str = include_str!("create_db.sql");

fn main() {
    let db = establish_connection();

    match db.batch_execute(EMPTY_DB_SQL) {
        Ok(_) => (),
        Err(_) => (),
    }
    match db.batch_execute(CREATE_DB_SQL) {
        Ok(_) => (),
        Err(err) => println!("Failed to create DB: {}", err),
    }
}
