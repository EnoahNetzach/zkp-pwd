use crate::client_data::ClientData;
use crate::database::DB;
use crate::web_utils::{DbTxErr, JsonErr};
use ntex::http::{Response as HttpResponse, StatusCode};
use ntex::web;
use ntex::web::DefaultError;
use serde::Serialize;
use serde_json::from_slice;
use sled::transaction::TransactionResult;
use std::borrow::Borrow;

#[derive(Debug, Serialize)]
struct Response {
    auth: bool,
}

impl Response {
    pub fn new(auth: bool) -> Self {
        Self { auth }
    }
}

fn do_authenticated(client_id: &str) -> Result<Response, DbTxErr> {
    let tx_res: TransactionResult<bool, DbTxErr> = DB.lock().unwrap().transaction(|tx_db| {
        let mut data: ClientData =
            from_slice(tx_db.get(client_id).unwrap().unwrap().borrow()).unwrap();

        let auth = !data.should_continue();

        data.auth = Some(auth);
        data.tests = vec![];

        tx_db
            .insert(
                client_id.clone(),
                serde_json::to_string(&data).unwrap().as_str(),
            )
            .unwrap();

        Ok(auth)
    });

    Ok(Response::new(tx_res.unwrap()))
}

#[web::get("")]
async fn authenticated(req: web::HttpRequest) -> Result<HttpResponse, web::Error> {
    let client_id = req
        .headers()
        .get("x-client-id")
        .unwrap()
        .to_str()
        .or(Err(JsonErr::new(
            StatusCode::BAD_REQUEST,
            "x-client-id header not present or not valid",
        )))?;

    Ok(
        HttpResponse::Ok().json(&do_authenticated(client_id).or(Err(JsonErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "unable to authenticate",
        )))?),
    )
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/authenticated").service(authenticated)
}
