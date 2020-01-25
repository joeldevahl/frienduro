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
use actix_web::{
    get, http, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use frienduro::{create_user, get_event, get_events, get_user, get_users};
use futures::Future;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

fn handler_get_users(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
) -> HttpResponse {
    let conn = db.get().unwrap();

    let users = get_users(&conn).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&users).unwrap())
}

fn handler_create_user(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
    item: web::Json<frienduro::User>,
) -> HttpResponse {
    let conn = db.get().unwrap();
    let user = create_user(&conn, &item.0.name, &item.0.email).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&user).unwrap())
}

#[get("/api/events")]
fn handler_get_events(
    req: HttpRequest,
    db: web::Data<Pool<PostgresConnectionManager>>,
) -> HttpResponse {
    let conn = db.get().unwrap();

    let events = get_events(&conn).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&events).unwrap())
}

#[get("/api/events/{id}")]
fn handler_get_event(
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

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .service(
                web::resource("/api/users")
                    .route(web::post().to(handler_create_user))
                    .route(web::get().to(handler_get_users)),
            )
            .service(handler_get_events)
            .service(handler_get_event)
    })
    .bind("127.0.0.1:8088");

    match server {
        Ok(srv) => srv.run(),
        Err(err) => panic!(err.to_string()),
    };
}
