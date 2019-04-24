#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate dotenv;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

extern crate frienduro;

use dotenv::dotenv;
use std::env;
use std::fmt::Write;

use actix_web::http::Method;
use actix_web::{server, App, HttpRequest, HttpResponse};
use frienduro::{get_event, get_user};
use r2d2::Pool;
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

    let user = get_user(&db, uid).unwrap();

    let result: String = user.name;

    HttpResponse::Ok().content_type("text/plain").body(result)
}

fn event(req: &HttpRequest) -> HttpResponse {
    let db = DB_POOL.get().unwrap();

    let eid = req.match_info().get("id").unwrap().parse::<i64>().unwrap();

    let event = get_event(&db, eid).unwrap();

    let mut results = format!("Results for {}:\n", event.name);
    for (i, r) in event.results.iter().enumerate() {
        write!(
            &mut results,
            "{} {} - {} seconds\n",
            i + 1,
            r.username,
            r.time,
        )
        .unwrap();
    }

    HttpResponse::Ok().content_type("text/plain").body(results)
}

fn main() {
    dotenv().ok();

    server::new(|| {
        App::new()
            .resource("/user/{id}", |r| r.method(Method::GET).f(user))
            .resource("/event/{id}", |r| r.method(Method::GET).f(event))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
