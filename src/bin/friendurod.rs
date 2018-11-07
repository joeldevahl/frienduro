#[macro_use]
extern crate lazy_static;
extern crate dotenv;
extern crate actix_web;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

extern crate frienduro;

use dotenv::dotenv;
use std::env;
use std::fmt::Write;

use actix_web::{server, App, HttpRequest, HttpResponse};
use actix_web::http::{Method};
use r2d2::{Pool};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

pub fn create_db_pool() -> Pool<PostgresConnectionManager> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = PostgresConnectionManager::new(database_url, TlsMode::None).unwrap();
    Pool::<PostgresConnectionManager>::new(manager).unwrap()
}

lazy_static! {
    pub static ref DB_POOL: Pool<PostgresConnectionManager> = create_db_pool();
}

fn user(req: &HttpRequest) -> HttpResponse {
    let db = DB_POOL.get().unwrap();

    let uid = req.match_info().get("id").unwrap().parse::<i64>().unwrap();

    let user_rows = db.query("SELECT * FROM users WHERE id = $1", &[&uid])
        .unwrap();

    let result: String  = user_rows.get(0).get("name");

    HttpResponse::Ok()
        .content_type("text/plain")
        .body(result)
}

fn event(req: &HttpRequest) -> HttpResponse {

    let db = DB_POOL.get().unwrap();

    let eid = req.match_info().get("id").unwrap().parse::<i64>().unwrap();

    let event_rows = db.query("SELECT * FROM events WHERE id = $1", &[&eid])
        .unwrap();
    let participation_rows = db.query("SELECT * FROM participations WHERE (event_id = $1 AND total_elapsed_seconds IS NOT NULL) ORDER BY total_elapsed_seconds DESC", &[&eid])
        .unwrap();

    let event_name: String = event_rows.get(0).get("name");
    let mut results = format!("Results for {}:\n", event_name);
    for (i, participation_row) in participation_rows.iter().enumerate() {
        let uid: i64 = participation_row.get("user_id");
        let user_rows = db.query("SELECT * FROM users WHERE id = $1", &[&uid])
            .unwrap();
        let user_name: String = user_rows.get(0).get("name");
        let seconds: i64 = participation_row.get("total_elapsed_seconds");
        
        write!(&mut results, "{} {} - {} seconds\n", i + 1, user_name, seconds);
    }

    HttpResponse::Ok()
    .content_type("text/plain")
    .body(results)
}

fn main() {
    dotenv().ok();

    server::new(|| App::new()
            .resource("/user/{id}", |r| r.method(Method::GET).f(user))
            .resource("/event/{id}", |r| r.method(Method::GET).f(event))
        )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}