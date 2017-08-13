extern crate simplenduro;
use self::simplenduro::establish_connection;

const EMPTY_DB_SQL: &'static str = include_str!("empty_db.sql");

fn main() {
    let db = establish_connection();

    db.batch_execute(EMPTY_DB_SQL).unwrap();
}
