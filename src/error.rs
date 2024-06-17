use std::fmt::write;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;


#[derive(Clone, Debug, Serialize)]
pub enum Error {
    FailToCreateCon,
    ErrorGetInfoFromDb,


    FailedHttpGetParking,
    FailedParsedBodyHttp,
    FailedParseToJson,
    FailedChangeTypeInJson,
    FailedToReadFile,



    FailedCreateAnyParking,
    FailedFoundAnyParking,
    FailedFoundCenter,
    FailedConvertType,
    AuthFailNoAuthTokenCookie,
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);

        response
    }
}