use std::time::Duration;

//use chrono::{DateTime, Utc};
//use serde_json;

use actix_web::{
    dev, get, http, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

use actix_web_opentelemetry::RequestMetrics;
use opentelemetry::global;

use opentelemetry::{sdk::Resource, KeyValue};
use opentelemetry_prometheus::PrometheusExporter;
use tracing_subscriber::Registry;
use tracing_subscriber::prelude::*;

fn init_meter() -> PrometheusExporter {
    opentelemetry_prometheus::exporter()
        .with_resource(Resource::new(vec![KeyValue::new("service", "users-api")]))
        .init()
}

static mut USERS_LIST: Vec<User> = Vec::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let exporter = init_meter();
    let meter = global::meter("users-api");

    // Optional predicate to determine which requests render the prometheus metrics
    let metrics_route =
        |req: &dev::ServiceRequest| req.path() == "/metrics" && req.method() == http::Method::GET;

    // Request metrics middleware
    let request_metrics = RequestMetrics::new(meter, Some(metrics_route), Some(exporter));

    Registry::default()
        .with(tracing_subscriber::EnvFilter::new("INFO"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    HttpServer::new(move || {
        App::new()
            .wrap(request_metrics.clone())
            .service(hello)
            .service(create_users)
            .service(get_users)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    std::thread::sleep(Duration::from_secs(1));
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    name: String,
    //login: String,
    age: i32,
    //created_at: DateTime<Utc>,
}

// create a new user
#[post("/users")]
pub async fn create_users(user: web::Json<User>) -> HttpResponse {
    println!("{:#?}", user);
    let u = User {
        name: user.name.to_string(),
        age: user.age,
    };

    unsafe {
        if USERS_LIST.len() == 1000 {
            USERS_LIST.clear();
        }
        USERS_LIST.push(u);
    }

    HttpResponse::Created()
        .content_type(ContentType::plaintext())
        .insert_header(("X-Hdr", "sample"))
        .body(format!("{} was created", user.name))
}

#[get("/users")]
pub async fn get_users() -> HttpResponse {
    let mut r: Vec<&User> = vec![];
    unsafe {
        for u in &USERS_LIST {
            r.push(u);
        }
    }

    HttpResponse::Created()
        .content_type(ContentType::plaintext())
        .insert_header(("X-Custom-Header", "1"))
        .json(r)
}
