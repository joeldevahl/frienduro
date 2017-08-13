extern crate dotenv;
extern crate postgres;
extern crate postgis;
extern crate elementtree;
extern crate chrono;

use dotenv::dotenv;
use std::env;

use postgres::{Connection, TlsMode};
use postgis::ewkb;

pub mod gpx;
pub mod models;

pub fn establish_connection() -> Connection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    Connection::connect(database_url, TlsMode::None).unwrap()
}

/*
fn main()
{
	let db = establish_connection();

    for row in &db.query("SELECT id, name, email FROM users", &[]).unwrap() {
        let user = User {
            id: row.get(0),
            name: row.get(1),
            email: row.get(2)
        };
        println!("Found user {} (id {})", user.name, user.id);
    }

    let ls = gpx::parse_gpx("gpx/Barnrunda.gpx".to_string()).unwrap();

    let name = "Barnrunda";
    db.execute("INSERT INTO segments (name, geom) VALUES ($1, $2)",
                 &[&name, &ls]).unwrap();

    for row in &db.query("SELECT id, name, geom FROM segments", &[]).unwrap() {
        let segment = Segment {
            id: row.get(0),
            name: row.get(1),
            geom: row.get(2),
        };
        println!("Found user {} (id {})", segment.name, segment.id);
    }
}
*/
