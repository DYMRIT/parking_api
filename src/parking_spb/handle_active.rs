use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::header::HeaderMap;
use axum::{Json};
use bb8_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_cookies::cookie::CookieBuilder;
use tower_cookies::cookie::time::Duration;
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::AppState;
use crate::error::{Result, Error};


#[derive(Debug, Serialize, Deserialize)]
pub struct ParkingRecord {
    vehicleNumber: String,
    parkingId: u32,
    startTime: u64,
    endTime: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ParkingRecordTime {
    id: u32,
    vehicleNumber: String,
    parkingId: u32,
    startTime: u64,
    endTime: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ParkingForChangeTime {
    vehicleNumber: Option<String>,
    parkingId: Option<u32>,
    startTime: Option<u64>,
    endTime: Option<u64>,
}


impl ParkingRecordTime {
    async fn new(parking_record: &ParkingRecord, app_state: &AppState) -> Self {
        let mut con = app_state.redis.get().await.unwrap();
        let _id = con.incr("parking_id", 1).await.unwrap_or_else(|_| 0);
        Self {
            id: _id,
            vehicleNumber: parking_record.vehicleNumber.clone(),
            parkingId: parking_record.parkingId.clone(),
            startTime: parking_record.startTime.clone(),
            endTime: parking_record.endTime.clone(),
        }
    }
}


pub async fn active_get(
    State(app_state): State<AppState>,
    cookies: Cookies,
    body: Bytes,
) -> Result<Json<Vec<Value>>> {
    println!("->> {:<12} active_get", "GET ACTIVE PARKING INFO");

    let mut con = app_state.redis.get().await.unwrap();
    println!("from str: {:?}", serde_json::from_str::<Value>(r#""{\"id\":2,\"vehicleNumber\":\"1231114\",\"parkingId\":178,\"startTime\":1767456,\"endTime\":1424235346}""#).unwrap());

    if let Some(token) = cookies.get("token") {
        let mut con = app_state.redis.get().await.unwrap();
        let key = format!("{}:*", &token.value().to_string());
        let keys: Vec<String> = con.keys(&key).await.unwrap();
        println!("keys: {:?}", keys);
        let mut values: Vec<Value> = vec![];
        for key in keys {
            let data: String = con.get(key)
                .await
                .unwrap();
            println!("data: {}", &data);
            let data_json: Value = serde_json::from_str(&data).unwrap();
            println!("json: {:?}", &data_json);
            values.push(data_json);
        }
        return Ok(Json(values))
    } else {
        return Ok(Json(vec![]))
    }
}


pub async fn active_post(
    State(app_state): State<AppState>,
    cookies: Cookies,
    parking_record: std::option::Option<Json<ParkingRecord>>
) -> Result<Json<()>> {
    println!("->> {:<12} active_post", "POST ACTIVE PARKING");
    match parking_record {
        Some(Json(parking_record)) => {
            println!("parking: {:?}", &parking_record);
            let parking_record_new = ParkingRecordTime::new(&parking_record, &app_state).await;
            let _id = parking_record_new.id;
            println!("parking_id: {}", &_id);
            let parking_string = serde_json::to_string(&parking_record_new).unwrap();

            let mut con = app_state.redis.get().await.unwrap();

            if let Some(token_cookie) = cookies.get("token") {
                // if have token in cookie
                let token = token_cookie.value().to_string();
                let key = format!("{}:{}", &token, &_id);
                let _: () = match con.set(key, &parking_string).await {
                    Ok(res) => res,
                    Err(e) => {
                        println!("Error: {}", e);
                        return Err(Error::ErrorGetInfoFromDb)
                    }
                };
            } else {
                // if haven't token in cookie
                let token = Uuid::new_v4().to_string();
                let key = format!("{}:{}", &token, &_id);
                let _: () = con.set(key, &parking_string).await.unwrap();

                let cookie = CookieBuilder::new("token", token)
                    .path("/")
                    .max_age(Duration::days(365))
                    .secure(false)
                    .http_only(true)
                    .finish();

                cookies.add(cookie);
            }

            Ok(Json({}))
        },
        None => Err(Error::FailedHttpGetParking)
    }
}


pub async fn active_delete(
    Path(id): Path<i64>,
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> Result<Json<()>> {
    println!("->> {:<12} active_delete", "DELETE ACTIVE PARKING");
    println!("id: {}", &id);
    if let Some(token) = cookies.get("token") {
        let mut con = app_state.redis.get().await.unwrap();
        let value = token.value().to_string();
        let key = format!("{}:{}", value, id);
        println!("key: {}", &key);
        match con.del::<&String, i64>(&key).await {
            Ok(_) => return Ok(Json({})),
            Err(e) => {
                println!("error: {}", e);
                return Err(Error::FailedDeleteDataFromDb)
            }
        }

    } else {
        return Err(Error::WrongCookie)
    }
}


pub async fn active_patch(
    Path(id): Path<i64>,
    cookies: Cookies,
    State(app_state): State<AppState>,
    payload: Option<Json<ParkingForChangeTime>>
) -> Result<Json<()>> {
    if let Some(token) = cookies.get("token") {
        let mut con = app_state.redis.get().await.unwrap();
        let value = token.value().to_string();
        let key = format!("{}:{}", value, id);
        let result = con.get::<&String, String>(&key).await;
        let mut parking_record: ParkingRecordTime = match result {
            Ok(parking_el_str) => {
                serde_json::from_str(&parking_el_str).unwrap()
            },
            Err(e) => return Err(Error::ErrorGetInfoFromDb)
        };
        if let Some(Json(payload)) = payload {
            match payload.startTime {
                Some(newStartTime) => {
                    parking_record.startTime = newStartTime
                },
                None => {}
            };
            match payload.endTime {
                Some(newEndTime) => {
                    parking_record.endTime = newEndTime
                },
                None => {}
            };
            let parking_string = serde_json::to_string(&parking_record).unwrap();
            let _: () = con.set(key, &parking_string).await.unwrap();
        } else {
            return Err(Error::FailedConvertType)
        }

    } else {
        return Err(Error::WrongCookie)
    }

    Ok(Json({}))
}