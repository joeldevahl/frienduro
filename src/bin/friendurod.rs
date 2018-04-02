#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate dotenv;
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;

extern crate frienduro;

use dotenv::dotenv;
use std::env;
use std::fmt;
use std::fmt::Write;
use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::{Success, Failure};
use rocket::http::Status;
use rocket::Request;
use diesel::PgConnection;
use diesel::prelude::*;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

use frienduro::models::*;

pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::<ConnectionManager<PgConnection>>::new(manager).unwrap()
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = create_db_pool();
}

pub struct DB(PooledConnection<ConnectionManager<PgConnection>>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DB {
    type Error = r2d2::Error;
    fn from_request(_: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Success(DB(conn)),
            Err(e) => Failure((Status::InternalServerError, e)),
        }
    }
}

fn load_user(conn: &PgConnection, uid:i64) -> Option<User> {
    use frienduro::schema::users::dsl::*;
    users
        .filter(id.eq(uid))
        .limit(1)
        .load::<User>(conn)
        .expect("Error loading user")
        .pop()
}

#[get("/user/<uid>")]
fn user(db: DB, uid: i64) -> String {
    let conn = db.conn();
    match load_user(conn, uid) {
        None => "".to_string(),
        Some(user) => format!("{}\n", user.name)
    }
}

#[get("/event/<eid>")]
fn event(db: DB, eid: i64) -> String {
    use frienduro::schema::events::dsl::*;
    use frienduro::schema::participations::dsl::*;

    let conn = db.conn();
    let event_rows = events
        .filter(frienduro::schema::events::dsl::id.eq(eid))
        .limit(1)
        .load::<Event>(conn)
        .expect("Error loading event");
    let participation_rows = participations
        .filter(event_id.eq(eid))
        .filter(total_elapsed_seconds.gt(0))
        .load::<Participation>(conn)
        .expect("Error loading participations");

    let mut results = format!("Results for {}:\n", event_rows[0].name);
    for (i, participation) in participation_rows.iter().enumerate() {
        let username = match load_user(conn, participation.user_id) {
            None => "Anonymous".to_string(),
            Some(user) => user.name
        };
        let seconds: i64 = participation.total_elapsed_seconds;
        
        write!(&mut results, "{} {} - {} seconds\n", i + 1, username, seconds);
    }

    return results;
}

fn main() {
    dotenv().ok();

    rocket::ignite().mount("/", routes![user, event]).launch();
}