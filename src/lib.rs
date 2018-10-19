extern crate dotenv;
extern crate postgres;
extern crate postgis;
extern crate elementtree;
extern crate chrono;

use dotenv::dotenv;
use std::env;

use postgres::{Connection, TlsMode};

pub mod gpx;

pub fn establish_connection() -> Connection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    Connection::connect(database_url, TlsMode::None).unwrap()
}