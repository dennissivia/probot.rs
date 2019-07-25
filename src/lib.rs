#[macro_use]
extern crate json;

use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use bytes::BytesMut;
use futures::{Future, Stream};
use json::JsonValue;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

/// This handler uses json extractor
fn index(item: web::Json<MyObj>) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

fn github_events(item: web::Json<MyObj>) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

// return a nice page or just redirect somewhere? Dunno...
// extract installation_ida and setup_action from
// https://probot-rust.ngrok.io/github/setup?installation_id=1326682&setup_action=install
fn github_setup() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("Setup successful. Imagine nice html")
}

pub fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(409600)) // <- limit size of the payload (global configuration)
            .service(web::resource("/github/setup").route(web::get().to(github_setup)))
            .service(web::resource("/github/events").route(web::post().to(github_events)))
            .service(web::resource("/").route(web::post().to(index)))
    })
    .bind("127.0.0.1:9999")?
    .run()
}
