use ntex::web;
use ntex::web::{DefaultError, HttpResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Response {
    ok: bool,
}

impl Response {
    pub fn new(ok: bool) -> Self {
        Self { ok }
    }
}

#[web::get("")]
async fn healthcheck() -> HttpResponse {
    HttpResponse::Ok().json(&Response::new(true))
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/healthcheck").service(healthcheck)
}
