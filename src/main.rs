mod error;
mod model;
mod parking_spb;

use std::collections::HashSet;
use std::io::Write;
use std::net::SocketAddr;
use axum::extract::State;
use axum::{Extension, http, Json, Router};
use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use bb8_redis::bb8::Pool;
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use http::Request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::ServiceExt;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_cookies::cookie::CookieBuilder;
use tower_cookies::cookie::time::Duration;
use tower_http::cors;
use tower_http::cors::{Any, Cors, CorsLayer};
use uuid::Uuid;
use error::{Result, Error};
use crate::parking_spb::api::send_http_to_parking;
use crate::parking_spb::construct::construct;
use crate::parking_spb::handle_active::{active_delete, active_get, active_patch, active_post};
use crate::parking_spb::parking_all::handler_parking;
use crate::parking_spb::parking_closest::handler_search_closest;
use crate::parking_spb::parking_fix::handler_fix_parking;


#[tokio::main]
async fn main() -> Result<()> {
    let app_state = AppState::new().await.unwrap();

    let client = reqwest::Client::new();
    match send_http_to_parking(&client, -1).await {
        Ok(parking) => {
            match construct(parking) {
                Ok(new_parking) => {
                    match write_to_file("parking.json", &new_parking) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("failed. Error: {}", "Failed write parking to file");
                            std::process::exit(1);
                        }
                    };
                },
                Err(e) => {
                    println!("failed. Error: {:?}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e)=> {
            println!("failed. Error: {:?}", e);
            std::process::exit(1);
        }
    };


    let route = Router::new()
        .route("/", get(hello_world))
        .route("/parkings", get(handler_parking))
        .route("/parkings/:id", get(handler_fix_parking))
        .route("/parkings/closest", get(handler_search_closest))
        .route("/parkings/active", get(active_get))
        .route("/parkings/active", post(active_post))
        .route("/parkings/active/:id", delete(active_delete))
        .route("/parkings/active/:id", patch(active_patch))
        .layer(CookieManagerLayer::new())
        .layer(axum::middleware::from_fn(dynamic_cors_layer))
        .layer(axum::middleware::map_response(map_response))
        //.layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127,0,0,1], 8081));
    println!("->> LISTENING on {addr}");
    axum::Server::bind(&addr)
        .serve(route.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn write_to_file(filename: &str, values: &Vec<Value>) -> std::io::Result<()> {
    let json = serde_json::to_string(values)?;
    let mut file = std::fs::File::create(filename)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}


#[derive(Clone, Debug)]
pub struct AppState {
    pub redis: RedisPool
}

impl AppState {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            redis: connect().await,
        })
    }
}

pub type RedisPool = Pool<RedisConnectionManager>;
async fn connect() -> RedisPool {
    let redis_manager = RedisConnectionManager::new("redis://127.0.0.1:6379").unwrap();
    bb8_redis::bb8::Pool::builder()
        .build(redis_manager)
        .await
        .unwrap()
}


async fn hello_world(body: Bytes) -> Result<Json<Value>> {
    println!("->> {:<12} hello_world", "HELLO_WORLD_HANDLER");

    Ok(Json(json!({
        "status": "ok"
    })))
}


async fn dynamic_cors_layer<B>(
    req: Request<B>,
    next: Next<B>
) -> Result<Response> {
    let origin_header = req.headers().get("Origin").cloned();

    let cors_origins = match origin_header {
        Some(origin) => vec![
            origin,
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://api.dvij24.ru"),
            HeaderValue::from_static("https://api.dvij24.ru"),
            HeaderValue::from_static("http://localhost:8081"),
        ],
        None => vec![
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://api.dvij24.ru"),
            HeaderValue::from_static("https://api.dvij24.ru"),
            HeaderValue::from_static("http://localhost:8081"),
        ],
    };

    let cors_layer = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            http::header::CONTENT_TYPE,
            HeaderName::from_static("ngrok-skip-browser-warning"),
        ])
        .allow_credentials(true);

    let svc = tower::ServiceBuilder::new()
        .layer(cors_layer)
        .service(next);

    match svc.oneshot(req).await {
        Ok(res) => Ok(res),
        Err(_) => Err(Error::FailToCreateCon)
    }

}


async fn map_response(
    headers: HeaderMap,
    res: Response) -> Response {
    println!("->> {:<12} map_response\n", "MAPPER RESPONSE LAST STEP");
    println!("headers: {:?}", &headers);
    let service_error = res.extensions().get::<Error>();

    match service_error {
        Some(err) => {
            match err {
                Error::ErrorGetInfoFromDb => return (StatusCode::OK, "Failed with cookie".to_string()).into_response(),
                _ => return (StatusCode::OK, Json(json!({"status": "error", "message": "Some failed"}))).into_response()
            }
        },
        None => return res
    }
}


// async fn set_db(key: &str, value: &str, pool: RedisPool) {
//     let mut connection = pool.get().await.unwrap();
//     let _: ()  = connection.set(key, value).await.unwrap();
// }
//
//
// async fn get_db(key: &str, pool: RedisPool) -> Option<String> {
//     let mut connection = pool.get().await.unwrap();
//     connection.get(key).await.unwrap()
// }


