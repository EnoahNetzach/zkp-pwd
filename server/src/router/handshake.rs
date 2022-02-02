use crate::client_data::ClientData;
use crate::database::DB;
use crate::web_utils::JsonErr;
use ntex::http::{Response as HttpResponse, StatusCode};
use ntex::web;
use ntex::web::DefaultError;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use pwd_dl_zkp_victor::victor::Victor;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Response {
    #[serde(rename = "clientId")]
    client_id: String,
    p: String,
    g: String,
}

impl Response {
    pub fn new(client_id: &str, p: &str, g: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            p: p.to_string(),
            g: g.to_string(),
        }
    }
}

fn do_handshake() -> Result<Response, ErrorStack> {
    let mut client_id = BigNum::new()?;
    client_id.rand(128, MsbOption::MAYBE_ZERO, false)?;
    let client_id = client_id.to_hex_str()?.to_string().to_lowercase();

    let victor = Victor::new();
    let (p, g) = victor.handshake()?;
    let p = p.to_hex_str()?.to_string();
    let g = g.to_hex_str()?.to_string();

    let mut data = ClientData::new();
    data.p = Some(p.clone());
    data.g = Some(g.clone());

    DB.lock()
        .unwrap()
        .insert(
            client_id.clone(),
            serde_json::to_string(&data).unwrap().as_str(),
        )
        .unwrap();

    Ok(Response::new(client_id.as_str(), p.as_str(), g.as_str()))
}

#[web::get("")]
async fn handshake() -> Result<HttpResponse, web::Error> {
    Ok(HttpResponse::Ok().json(&do_handshake().or(Err(JsonErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "unable to handshake",
    )))?))
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/handshake").service(handshake)
}
