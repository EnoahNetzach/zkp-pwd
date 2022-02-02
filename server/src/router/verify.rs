use crate::client_data::{ClientData, ClientTest};
use crate::database::DB;
use crate::web_utils::{DbTxErr, JsonErr};
use ntex::http::{Response as HttpResponse, StatusCode};
use ntex::web;
use ntex::web::DefaultError;
use openssl::bn::BigNum;
use pwd_dl_zkp_core::core::Choice;
use pwd_dl_zkp_victor::victor::Victor;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use sled::transaction::TransactionResult;
use std::borrow::Borrow;

#[derive(Debug, Deserialize)]
struct Request {
    res: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Response {
    cont: bool,
    valid: bool,
}

impl Response {
    pub fn new(
        choice: &Choice,
        res: &BigNum,
        c: &BigNum,
        y: &BigNum,
        g: &BigNum,
        p: &BigNum,
        executions: usize,
    ) -> Self {
        let victor = Victor::new();
        let valid = victor.verify(choice, res, c, y, g, p).unwrap();
        let executions = executions + if valid { 1 } else { 0 };

        Self {
            cont: executions < 10,
            valid,
        }
    }
}

fn do_verify(client_id: &str, res: &BigNum) -> Result<Response, DbTxErr> {
    let tx_res: TransactionResult<Response, DbTxErr> = DB.lock().unwrap().transaction(|tx_db| {
        let mut data: ClientData =
            from_slice(tx_db.get(client_id).unwrap().unwrap().borrow()).unwrap();

        let mut client_test = data.tests.pop().unwrap();

        let (choice, c, y, g, p) = match (data.clone(), client_test.clone()) {
            (
                ClientData {
                    g: Some(g),
                    p: Some(p),
                    y: Some(y),
                    ..
                },
                ClientTest {
                    c: Some(c),
                    choice: Some(choice),
                    valid: None,
                    ..
                },
            ) => Ok((choice, c, y, g, p)),
            _ => sled::transaction::abort(DbTxErr::new("error cloning client data")),
        }?;

        let (c, y, g, p) = match (
            BigNum::from_hex_str(c.as_str()),
            BigNum::from_hex_str(y.as_str()),
            BigNum::from_hex_str(g.as_str()),
            BigNum::from_hex_str(p.as_str()),
        ) {
            (Ok(c), Ok(y), Ok(g), Ok(p)) => Ok((c, y, g, p)),
            _ => sled::transaction::abort(DbTxErr::new("error extracting client data")),
        }?;

        let valid_tests = data
            .tests
            .iter()
            .filter(|t| t.valid.unwrap_or(false))
            .count();
        let v = Response::new(&choice, &res, &c, &y, &g, &p, valid_tests);

        client_test.valid = Some(v.valid);

        data.tests.push(client_test.clone());
        tx_db
            .insert(
                client_id.clone(),
                serde_json::to_string(&data).unwrap().as_str(),
            )
            .unwrap();

        Ok(v)
    });

    Ok(tx_res.unwrap())
}

#[web::post("")]
async fn verify(
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

    let res = BigNum::from_hex_str(&data.res).or(Err(JsonErr::new(
        StatusCode::BAD_REQUEST,
        "res is not a number",
    )))?;

    Ok(
        HttpResponse::Ok().json(&do_verify(client_id, &res).or(Err(JsonErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "unable to verify",
        )))?),
    )
}

pub(crate) fn routes() -> web::Scope<DefaultError> {
    web::scope("/verify").service(verify)
}
