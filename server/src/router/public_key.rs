use crate::client_data::ClientData;
use crate::database::DB;
use crate::web_utils::{DbTxErr, JsonErr};
use ntex::http::{Response as HttpResponse, StatusCode};
use ntex::web;
use ntex::web::DefaultError;
use openssl::bn::BigNum;
use serde::Deserialize;
use serde_json::from_slice;
use sled::transaction::TransactionResult;
use std::borrow::Borrow;

#[derive(Debug, Deserialize)]
struct Request {
    y: String,
}

fn do_public_key(client_id: &str, y: &str) -> Result<(), DbTxErr> {
    let tx_res: TransactionResult<(), DbTxErr> = DB.lock().unwrap().transaction(|tx_db| {
        let mut data: ClientData =
            from_slice(tx_db.get(client_id).unwrap().unwrap().borrow()).unwrap();

        data.y = Some(String::from(y));

        tx_db
            .insert(
                client_id.clone(),
                serde_json::to_string(&data).unwrap().as_str(),
            )
            .unwrap();

        Ok(())
    });
    tx_res.unwrap();

    Ok(())
}

#[web::post("")]
async fn public_key(
    req: web::HttpRequest,
    data: web::types::Json<Request>,
) -> Result<HttpResponse, web::Error> {
    let client_id = req
        .headers()
        .get("x-client-id")
        .unwrap()
        .to_str()
        .or(Err(JsonErr::new(
            StatusCode::BAD_REQUEST,
            "x-client-id header not present or not valid",
        )))?;

    BigNum::from_hex_str(&data.y).or(Err(JsonErr::new(
        StatusCode::BAD_REQUEST,
        "y is not a number",
    )))?;

    Ok(
        HttpResponse::Ok().json(&do_public_key(client_id, data.y.as_str()).or(Err(
            JsonErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to process public key",
            ),
        ))?),
    )
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/public-key").service(public_key)
}
