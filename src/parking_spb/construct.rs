use std::fmt::format;
use std::ptr::eq;
use serde_json::{json, Value};
use crate::error::{Result, Error};


pub fn construct(parking: Value) -> Result<Vec<Value>> {
    let vec_parking = match parking.as_array() {
        Some(res) => res,
        None => return Err(Error::FailedChangeTypeInJson),
    };
    let mut new_vec_parking: Vec<Value> = vec![];
    for parking in vec_parking {
        let parking_after_construct = match construct_one_parking(parking) {
            Ok(parking) => parking,
            Err(e) => return Err(e)
        };
        new_vec_parking.push(parking_after_construct);
    }
    Ok(new_vec_parking)
}


pub fn construct_one_parking(parking: &Value) -> Result<Value> {
    let id = &parking["_id"];
    let location = &parking["location"];
    let location = &parking["location"];
    let center = &parking["center"]["coordinates"];
    let house = &parking["address"]["house"]["ru"];
    let street = &parking["address"]["street"]["ru"];
    let total_all_parking = &parking["spaces"]["total"];
    let total_basic_parking = &parking["spaces"]["common"];
    let total_handicapped_parking = &parking["spaces"]["handicapped"];
    let available_basic_parking = &parking["congestion"]["spaces"]["overall"]["free"];
    let zone = &parking["zone"]["number"];
    let created_at = chrono::Local::now().timestamp();

    let prices = &parking["zone"]["prices"];

    let prices = match prices.as_array() {
        Some(arr) => arr.clone(),
        None => vec![json!({})]
    };
    let mut new_prices = json!({});
    for price in prices {
        let vehicle_type = price["vehicleType"].as_str().unwrap_or_else(|| "");
        let price_min = &price["price"]["min"];
        let price_max = &price["price"]["max"];
        let new_field = json!({
            "min": price_min,
            "max": price_max
        });
        if let Some(map) = new_prices.as_object_mut() {
            map.insert(vehicle_type.to_string(), new_field);
        }
        // let obj = json!({
        //     vehicle_type.to_string(): {
        //         "min": price_min,
        //         "max": price_max
        //     }
        // });
        // new_prices.push(obj)
    }

    let total = total_all_parking.as_i64().unwrap_or_else(|| -1);
    let basic = total_basic_parking.as_i64().unwrap_or_else(|| -1);
    let handicapped = total_handicapped_parking.as_i64().unwrap_or_else(|| -1);
    let available = available_basic_parking.as_i64().unwrap_or_else(|| -1);

    let basic_parking: Value = if available != -1 && total != -1 && handicapped != -1 {
        json!({
                "free": available,
                "total": total - handicapped,
            })
    } else if available != -1 && total != -1 {
        json!({
                "free": available,
                "total": total,
            })
    } else if available == -1 && (total != -1 && handicapped != -1 && (total - handicapped) != 0){
        json!({
                "free": total - handicapped,
                "total": total - handicapped,
            })
    } else if (basic == 0 && handicapped != -1) || (handicapped != -1 && total != -1 && (total - handicapped == 0)) {
        json!({
                "free": 0,
                "total": 0,
            })
    } else if total != -1 && basic != -1 && (total - basic == 0) {
        json!({
                "free": total,
                "total": basic,
            })
    } else if total != -1 && basic == -1 && handicapped == -1 && available == -1 {
        json!({
                "free": total,
                "total": total,
            })
    } else if parking["spaces"].is_null() {
        json!({
                "free": "None",
                "total": "None",
            })
    } else if parking["spaces"].as_object().unwrap().is_empty() {
        json!({
                "free": "None",
                "total": "None",
            })
    } else {
        return Err(Error::FailedCreateAnyParking);
    };

    let type_parking = check_handicapped(total, basic, handicapped);

    if !id.is_null() && !center.is_null() &&
        !location.is_null() && !street.is_null() && !house.is_null() {

        let obj = json!({
                "created_at": created_at,
                "id": id,
                "type": type_parking,
                "center": center,
                "price": new_prices,
                "location": location,
                "address": {
                    "street": street,
                    "house": house,
                },
                "spaces": {
                    "common": basic_parking,
                    "handicapped": handicapped,
                },
                "zone": {
                    "number": zone
                },
            });
        return Ok(obj)
    } else {
        return Err(Error::FailedCreateAnyParking);
    }
}


fn check_handicapped(total: i64, car: i64, handicapped: i64) -> String {
    if (car > 0) || (car == total) && (total >= 0) || (handicapped == 0) {
        return "common".to_string()
    } else if (handicapped == total) {
        return "handicapped".to_string()
    } else {
        return "None".to_string()
    }
    // if car != -1 || ((total - handicapped) != 0 && total != -1 && handicapped != -1) {
    //     "common".to_string()
    // } else if total - handicapped == 0 && total != -1 && handicapped != -1 {
    //     "handicapped".to_string()
    // } else {
    //     "None".to_string()
    // }
}