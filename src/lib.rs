extern crate dotenv;
extern crate postgres;
extern crate postgis;
#[macro_use]
extern crate diesel;
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
pub mod models
{
    #[derive(Queryable)]
    pub struct User {
        pub id: i64,
        pub name: String,
        pub email: String,
    }

    #[derive(Queryable)]
    pub struct Event {
        pub id: i64,
        pub name: String,
    }

    #[derive(Queryable)]
    pub struct Participation {
        pub id: i64,
        pub event_id: i64,
        pub user_id: i64,
        pub route_id: i64,
        pub source_id: i64,
        pub total_elapsed_seconds: i64,
    }
}

use models::*;

pub mod schema
{
    table! {
        events (id) {
            id -> Int8,
            name -> Varchar,
        }
    }

    table! {
        participations (id) {
            id -> Int8,
            event_id -> Int8,
            user_id -> Int8,
            route_id -> Int8,
            source_id -> Int8,
            total_elapsed_seconds -> Int8,
        }
    }

    table! {
        users (id) {
            id -> Int8,
            name -> Varchar,
            email -> Varchar,
        }
    }

    joinable!(participations -> events (event_id));
    joinable!(participations -> users (user_id));

    allow_tables_to_appear_in_same_query!(
        events,
        participations,
        users,
    );
}