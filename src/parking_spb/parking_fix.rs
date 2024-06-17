use axum::body::Bytes;
use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};
use crate::error::{Result, Error};
use crate::parking_spb::construct::construct_one_parking;

pub async fn handler_fix_parking(Path(id): Path<i64>, body: Bytes) -> Result<Json<Value>> {
    println!("->> {:<12}", "HANDLER_ANY_PARKING");
    let url = format!("https://parking.spb.ru/api/2.71/parkings/{id}");
    let response = match reqwest::get(&url).await {
        Ok(res) => res,
        Err(e) => return Err(Error::FailedHttpGetParking)
    };
    let text_response = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(Error::FailedParsedBodyHttp)
    };
    let json: Value = match serde_json::from_str(&text_response) {
        Ok(json) => json,
        Err(e) => return Err(Error::FailedParseToJson)
    };
    let parking: Value = match json.get("parking") {
        Some(parking) => parking.clone(),
        None => json!({}),
    };
    let new_parking = match construct_one_parking(&parking) {
        Ok(res) => res,
        Err(e) => return Err(e)
    };

    Ok(Json(new_parking))
}