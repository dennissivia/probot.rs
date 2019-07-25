#[macro_use]
extern crate json;

use actix_web::http::header::HeaderMap;
use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use bytes::BytesMut;
use futures::{Future, Stream};
use json::JsonValue;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CheckRun {
    id: u32,
    node_id: String,
    head_sha: String,
    status: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct CheckRunPayload {
    name: String,
    number: i32,
    check_run: CheckRun,
}
const MAX_SIZE: usize = 262_144; // max payload size is 256k

fn event_header_value(headers: &HeaderMap) -> Option<String> {
    let maybeHeader = headers.get("X-GITHUB-EVENT");
    match maybeHeader {
        Some(header) => {
            println!("{:?}", header);
        }
        None => {
            println!("Did not find Event header to parse data");
        }
    }
    maybeHeader.and_then(|header| header.to_str().ok().map(String::from))
}

fn extract_payload(payload: web::Payload) -> impl Future<Item = CheckRunPayload, Error = Error> {
    payload
        // `Future::from_err` acts like `?` in that it coerces the error type from
        // the future into the final error type
        .from_err()
        // `fold` will asynchronously read each chunk of the request body and
        // call supplied closure, then it resolves to result of closure
        .fold(BytesMut::new(), move |mut body, chunk| {
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        // `Future::and_then` can be used to merge an asynchronous workflow with a
        // synchronous workflow
        .and_then(|body| {
            // body is loaded, now we can deserialize serde-json
            let obj = serde_json::from_slice::<CheckRunPayload>(&body)?;
            Ok(obj)
            // Ok(HttpResponse::Ok().json(obj)) // <- send response
        })
}

// fn github_events(item: web::Json<CheckRunPayload>) -> HttpResponse {
// fn github_events(req: HttpRequest, pl: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
fn github_events(req: HttpRequest, payload: web::Payload) -> HttpResponse {
    let event_type = event_header_value(req.headers());

    match event_type {
        Some(header) => {
            match header.as_str() {
                "check_suite" => {
                    println!("{:?}", &req);
                    // println!("model: {:?}", payload);
                    let data = extract_payload(payload).wait();
                    println!("model: {:?}", data);
                    // .map(|obj| {
                    //       println!("{:?}", obj);
                    //       obj
                    //     })
                }
                _ => println!("Unknown event type: {:?}", header),
            }
        }
        None => {}
    }
    //    HttpResponse::Ok().json(item.0)
    HttpResponse::Ok().content_type("application/json").body("")

    // HttpResponse::Ok()
    //     .content_type("text/html")
    //     .body("Setup successful. Imagine nice html")
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
        //X-GitHub-Event
    })
    .bind("127.0.0.1:9999")?
    .run()
}
