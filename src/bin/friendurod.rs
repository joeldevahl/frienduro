extern crate actix_web;
extern crate dotenv;
extern crate futures;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

extern crate frienduro;

use dotenv::dotenv;
use std::env;
use std::fmt::Write;

use actix_web::http::Method;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use frienduro::{get_event, get_user};
use futures::Future;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

#[get("/user/{id}")]
fn user(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
    id: web::Path<i64>,
) -> HttpResponse {
    let conn = db.get().unwrap();

    let uid = id.into_inner();
    let user = get_user(&conn, uid).unwrap();

    HttpResponse::Ok().json(user);
}

#[get("/event/{id}")]
fn event(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
    id: web::Path<i64>,
) -> HttpResponse {
    let conn = db.get().unwrap();

    let eid = id.into_inner();

    let event = get_event(&conn, eid).unwrap();

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
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = PostgresConnectionManager::new(database_url, TlsMode::None).unwrap();
    let pool = Pool::<PostgresConnectionManager>::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .service(user)
            .service(event)
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
