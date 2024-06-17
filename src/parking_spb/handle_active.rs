use axum::body::Bytes;
use axum::extract::State;
use axum::Json;
use bb8_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use serde_json::{json, Value};
use tower_cookies::cookie::CookieBuilder;
use tower_cookies::cookie::time::Duration;
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::AppState;
use crate::error::{Result, Error};

pub async fn active_get(
    State(app_state): State<AppState>,
    cookies: Cookies,
    body: Bytes
) -> crate::error::Result<Json<Value>> {
    println!("->> {:<12} active_get", "GET ACTIVE PARKING INFO");

    if let Some(token) = cookies.get("token") {
        let mut con = app_state.redis.get().await.unwrap();
        let redis_data: String = match con.get(token.value().to_string().clone()).await {
            Ok(data) => data,
            Err(e) => {
                println!("error: {:?}", e);
                return Err(Error::ErrorGetInfoFromDb)
            }
        };
        let mut list_arr = serde_json::from_str::<Value>(&redis_data).unwrap();
        return Ok(Json(list_arr))
    } else {
        return Ok(Json(json!([])))
    }
}


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
            let parking_id = parking_record_new.parkingId;
            let parking_string = serde_json::to_string(&parking_record_new).unwrap();

            let mut con = app_state.redis.get().await.unwrap();

            if let Some(token_cookie) = cookies.get("token") {
                // if have token in cookie
                let token = token_cookie.value().to_string();
                println!("token: {:?}", &token);
                let redis_data: String = match con.get(token.clone()).await {
                    Ok(data) => data,
                    Err(e) => {
                        println!("error: {:?}", e);
                        return Err(Error::ErrorGetInfoFromDb)
                    }
                };
                let mut list_arr = serde_json::from_str::<Vec<String>>(&redis_data).unwrap();
                list_arr.push(parking_string.to_string());

                let _: () = match con.set(&token, serde_json::to_string(&list_arr).unwrap()).await {
                    Ok(res) => res,
                    Err(e) => {
                        println!("Error: {}", e);
                        return Err(Error::ErrorGetInfoFromDb)
                    }
                };
            } else {
                // if haven't token in cookie
                let token = Uuid::new_v4().to_string();
                let arr = serde_json::to_string(&vec![&parking_string]).unwrap();
                let _: () = con.set(&token, arr).await.unwrap();

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