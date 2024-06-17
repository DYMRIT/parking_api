use reqwest::Client;
use serde_json::{json, Value};
use crate::error::{Result, Error};


pub async fn send_http_to_parking(client: &Client, id: i64) -> Result<Value> {
    let url = if id == -1 {
        format!("https://parking.spb.ru/api/2.71/parkings/")
    } else {
        format!("https://parking.spb.ru/api/2.71/parkings/{id}")
    };
    let request = client.get(url);
    let response = match request.send().await {
        Ok(resp) => resp,
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
    let parking: Value = match json.get("parkings") {
        Some(parking) => parking.clone(),
        None => json!({}),
    };

    Ok(parking)
}