extern crate actix_web;
extern crate dotenv;
extern crate futures;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate serde_json;

extern crate frienduro;

use dotenv::dotenv;
use std::env;
use std::fmt::Write;

use actix_web::http::Method;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use frienduro::{get_event, get_events, get_user};
use futures::Future;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

#[get("/api/events")]
fn events(req: HttpRequest, db: web::Data<Pool<PostgresConnectionManager>>) -> HttpResponse {
    let conn = db.get().unwrap();

    let events = get_events(&conn).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&events).unwrap())
}

#[get("/api/events/{id}")]
fn events_with_id(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
    id: web::Path<i64>,
) -> HttpResponse {
    let conn = db.get().unwrap();

    let event = get_event(&conn, id.into_inner()).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&event).unwrap())
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = PostgresConnectionManager::new(database_url, TlsMode::None).unwrap();
    let pool = Pool::<PostgresConnectionManager>::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .service(events)
            .service(events_with_id)
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
