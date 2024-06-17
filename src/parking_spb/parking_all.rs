use axum::body::Bytes;
use axum::Json;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use crate::error::{Result, Error};

pub async fn handler_parking(body: Bytes) -> Result<Json<Vec<Value>>> {
    println!("->> {:<12}", "HANDLER_ALL_PARKING");
    let mut file = tokio::fs::File::open("parking.json").await.map_err(|_| Error::FailedToReadFile)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.map_err(|_| Error::FailedToReadFile)?;

    let parkings: Vec<Value> = serde_json::from_str(&contents).map_err(|_| Error::FailedParseToJson)?;
    Ok(Json(parkings))
}