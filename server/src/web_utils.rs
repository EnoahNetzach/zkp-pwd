use ntex::http::{Response, StatusCode};
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize)]
pub(crate) struct JsonErr {
    err: String,
    #[serde(skip_serializing)]
    status_code: StatusCode,
}

impl JsonErr {
    pub fn new(status_code: StatusCode, str: &str) -> Self {
        Self {
            err: String::from(str),
            status_code,
        }
    }
}

impl Display for JsonErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JsonError({:?})", &self.err)
    }
}

impl WebResponseError for JsonErr {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self, _: &HttpRequest) -> Response {
        HttpResponse::Ok().status(self.status_code).json(&self)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct DbTxErr {
    err: String,
}

impl DbTxErr {
    pub fn new(str: &str) -> Self {
        Self {
            err: String::from(str),
        }
    }
}
