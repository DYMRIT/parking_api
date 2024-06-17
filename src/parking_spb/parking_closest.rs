use axum::body::Bytes;
use axum::extract::Query;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::AsyncReadExt;
use crate::error::{Result, Error};


#[derive(Debug, Deserialize)]
pub struct Coordinates {
    lng: Option<String>,
    lat: Option<String>,
}


pub async fn handler_search_closest(Query(params): Query<Coordinates>, body: Bytes) -> Result<Json<Value>>{
    println!("->> {:<12}", "HANDLER_SEARCH_CLOSEST");
    println!("{:?}", params);
    let mut file = tokio::fs::File::open("parking.json").await.map_err(|_| Error::FailedToReadFile)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.map_err(|_| Error::FailedToReadFile)?;
    let parkings: Vec<Value> = serde_json::from_str(&contents).map_err(|_| Error::FailedParseToJson)?;

    let (center_1, center_2) = match (params.lng.as_deref(), params.lat.as_deref()) {
        (Some(lng), Some(lat)) => {
            match (lng.parse::<f64>(), lat.parse::<f64>()) {
                (Ok(lng), Ok(lat)) => (lng, lat),
                _ => return Err(Error::FailedConvertType)
            }
        },
        _ => return Err(Error::FailedConvertType),
    };

    let mut dist_min: f64 = 1000.0;
    let mut id_best: i64 = -1;



    for parking in parkings {
        let (center_new_1, center_new_2) = match convert_two_digit(&parking) {
            Ok((a1, a2)) => (a1,a2),
            Err(e) => return Err(e),
        };
        let dist_cur = euclidean_distance(center_1, center_2, center_new_1, center_new_2);
        if dist_cur < dist_min {
            dist_min = dist_cur;
            id_best = match parking["id"].as_i64() {
                Some(id) => id,
                None => return Err(Error::FailedConvertType)
            };
        }
    }
    Ok(Json(json!({"id": id_best})))
}


fn euclidean_distance(a1: f64, a2: f64, b1: f64, b2: f64) -> f64 {
    let dig = (a1 - b1).powf(2.0) + (a2 - b2).powf(2.0);
    dig.sqrt()
}


fn convert_two_digit(val: &Value) -> Result<(f64, f64)> {
    let center = match val.get("center") {
        Some(res) => {
            match res.as_array() {
                Some(res1) => res1,
                None => return Err(Error::FailedFoundCenter),
            }
        },
        None => return Err(Error::FailedFoundCenter),
    };
    if center.len() != 2 {
        return Err(Error::FailedFoundCenter)
    };
    let a1 = match center[0].as_f64() {
        Some(dig) => dig,
        None => return Err(Error::FailedConvertType)
    };
    let a2 = match center[1].as_f64() {
        Some(dig) => dig,
        None => return Err(Error::FailedConvertType)
    };
    Ok((a1, a2))
}