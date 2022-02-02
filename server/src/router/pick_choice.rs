use crate::client_data::{ClientData, ClientTest};
use crate::database::DB;
use crate::web_utils::{DbTxErr, JsonErr};
use ntex::http::{Response as HttpResponse, StatusCode};
use ntex::web;
use ntex::web::DefaultError;
use openssl::bn::BigNum;
use pwd_dl_zkp_core::core::Choice;
use pwd_dl_zkp_victor::victor::Victor;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use sled::transaction::TransactionResult;
use std::borrow::Borrow;

#[derive(Debug, Deserialize)]
struct Request {
    c: String,
}

#[derive(Debug, Serialize)]
struct Response {
    choice: Choice,
}

impl Response {
    pub fn new(choice: Choice) -> Self {
        Self { choice }
    }
}

fn do_pick_choice(client_id: &str, c: &str) -> Result<Response, DbTxErr> {
    let mut rng = thread_rng();
    let victor = Victor::new();
    let choice = victor.pick_choice(&mut rng);

    let tx_res: TransactionResult<(), DbTxErr> = DB.lock().unwrap().transaction(|tx_db| {
        let mut data: ClientData =
            from_slice(tx_db.get(client_id).unwrap().unwrap().borrow()).unwrap();

        let mut client_test = ClientTest::default();
        client_test.c = Some(String::from(c));
        client_test.choice = Some(choice);

        data.tests.push(client_test);

        tx_db
            .insert(
                client_id.clone(),
                serde_json::to_string(&data).unwrap().as_str(),
            )
            .unwrap();

        Ok(())
    });
    tx_res.unwrap();

    Ok(Response::new(choice))
}

#[web::post("")]
async fn pick_choice(
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

    BigNum::from_hex_str(&data.c).or(Err(JsonErr::new(
        StatusCode::BAD_REQUEST,
        "c is not a number",
    )))?;

    Ok(
        HttpResponse::Ok().json(&do_pick_choice(client_id, data.c.as_str()).or(Err(
            JsonErr::new(StatusCode::INTERNAL_SERVER_ERROR, "unable to pick a choice"),
        ))?),
    )
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/pick-choice").service(pick_choice)
}
