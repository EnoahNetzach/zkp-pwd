use lazy_static::lazy_static;
use ntex::web;
use ntex::web::{middleware, App, HttpResponse};
use ntex_cors::Cors;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use pwd_dl_zkp_core::core::Choice;
use pwd_dl_zkp_victor::victor::Victor;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use sled::transaction::TransactionResult;
use std::borrow::Borrow;
use std::time::SystemTime;

lazy_static! {
    static ref DB: sled::Db = sled::open("db/client_data").unwrap();
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ClientTest {
    pub c: Option<String>,
    pub choice: Option<Choice>,
    pub valid: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ClientData {
    pub created_at: SystemTime,
    pub p: Option<String>,
    pub g: Option<String>,
    pub y: Option<String>,
    pub tests: Vec<ClientTest>,
    pub auth: Option<bool>,
}

impl Default for ClientData {
    fn default() -> Self {
        Self {
            created_at: SystemTime::now(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonErr {
    err: String,
}

impl JsonErr {
    pub fn new(str: &str) -> Self {
        Self {
            err: String::from(str),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DbTxErr {
    err: String,
}

impl DbTxErr {
    pub fn new(str: &str) -> Self {
        Self {
            err: String::from(str),
        }
    }
}

#[derive(Debug, Serialize)]
struct Healthcheck {
    ok: bool,
}

impl Healthcheck {
    pub fn new(ok: bool) -> Self {
        Self { ok }
    }
}

#[web::get("/healthcheck")]
async fn healthcheck() -> HttpResponse {
    let hc = Healthcheck::new(true);

    HttpResponse::Ok().json(&hc)
}

#[derive(Debug, Serialize)]
struct HandShake {
    #[serde(rename = "clientId")]
    client_id: String,
    p: String,
    g: String,
}

impl HandShake {
    pub fn new(client_id: &str) -> Result<Self, ErrorStack> {
        let victor = Victor::new();
        let (p, g) = victor.handshake()?;

        Ok(Self {
            client_id: client_id.to_string(),
            p: p.to_hex_str().unwrap().to_string(),
            g: g.to_hex_str().unwrap().to_string(),
        })
    }
}

#[web::get("/handshake")]
async fn handshake() -> Result<HttpResponse, web::Error> {
    let mut client_id = BigNum::new().unwrap();
    client_id.rand(128, MsbOption::MAYBE_ZERO, false).unwrap();
    let client_id = client_id.to_hex_str().unwrap().to_string().to_lowercase();

    let hs = HandShake::new(client_id.as_str()).unwrap();

    let mut data = ClientData::default();
    data.p = Some(hs.p.clone());
    data.g = Some(hs.g.clone());

    DB.insert(
        client_id.clone(),
        serde_json::to_string(&data).unwrap().as_str(),
    )
    .unwrap();

    Ok(HttpResponse::Ok().json(&hs))
}

#[derive(Debug, Deserialize)]
struct PublicKey {
    y: String,
}

#[web::post("/public-key")]
async fn public_key(
    req: web::HttpRequest,
    pk: web::types::Json<PublicKey>,
) -> Result<HttpResponse, web::Error> {
    match (
        req.headers().get("x-client-id").unwrap().to_str(),
        BigNum::from_hex_str(&pk.y),
    ) {
        (Ok(""), Ok(_)) | (Err(_), _) => {
            let err = JsonErr::new("no client id present");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (_, Err(_)) => {
            let err = JsonErr::new("y is not a number");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (Ok(client_id), Ok(_)) => {
            let tx_res: TransactionResult<(), DbTxErr> = DB.transaction(|tx_db| {
                let mut data: ClientData =
                    serde_json::from_slice(tx_db.get(client_id).unwrap().unwrap().borrow())
                        .unwrap();

                data.y = Some(pk.y.clone());

                tx_db
                    .insert(
                        client_id.clone(),
                        serde_json::to_string(&data).unwrap().as_str(),
                    )
                    .unwrap();

                Ok(())
            });
            tx_res.unwrap();

            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[derive(Debug, Deserialize)]
struct CalcC {
    c: String,
}

#[derive(Debug, Serialize)]
struct PickChoice {
    choice: Choice,
}

impl PickChoice {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let victor = Victor::new();
        let choice = victor.pick_choice(&mut rng);

        Self { choice }
    }
}

#[web::post("/pick-choice")]
async fn pick_choice(
    req: web::HttpRequest,
    cc: web::types::Json<CalcC>,
) -> Result<HttpResponse, web::Error> {
    match (
        req.headers().get("x-client-id").unwrap().to_str(),
        BigNum::from_hex_str(&cc.c),
    ) {
        (Ok(""), Ok(_)) | (Err(_), _) => {
            let err = JsonErr::new("no client id present");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (_, Err(_)) => {
            let err = JsonErr::new("c is not a number");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (Ok(client_id), Ok(_)) => {
            let pc = PickChoice::new();

            let tx_res: TransactionResult<(), DbTxErr> = DB.transaction(|tx_db| {
                let mut data: ClientData =
                    serde_json::from_slice(tx_db.get(client_id).unwrap().unwrap().borrow())
                        .unwrap();

                let mut client_test = ClientTest::default();
                client_test.c = Some(cc.c.clone());
                client_test.choice = Some(pc.choice);

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

            Ok(HttpResponse::Ok().json(&pc))
        }
    }
}

#[derive(Debug, Deserialize)]
struct CalcChoice {
    res: String,
}

#[derive(Debug, Serialize)]
struct Verify {
    cont: bool,
    valid: bool,
}

impl Verify {
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

        Self {
            cont: executions < 10,
            valid,
        }
    }
}

#[web::post("/verify")]
async fn verify(
    req: web::HttpRequest,
    cc: web::types::Json<CalcChoice>,
) -> Result<HttpResponse, web::Error> {
    match (
        req.headers().get("x-client-id").unwrap().to_str(),
        BigNum::from_hex_str(&cc.res),
    ) {
        (Ok(""), Ok(_)) | (Err(_), _) => {
            let err = JsonErr::new("no client id present");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (_, Err(_)) => {
            let err = JsonErr::new("c is not a number");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        (Ok(client_id), Ok(res)) => {
            let tx_res: TransactionResult<Verify, DbTxErr> = DB.transaction(|tx_db| {
                let mut data: ClientData =
                    serde_json::from_slice(tx_db.get(client_id).unwrap().unwrap().borrow())
                        .unwrap();

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
                    _ => sled::transaction::abort(DbTxErr::new("")),
                }?;

                let (c, y, g, p) = match (
                    BigNum::from_hex_str(c.as_str()),
                    BigNum::from_hex_str(y.as_str()),
                    BigNum::from_hex_str(g.as_str()),
                    BigNum::from_hex_str(p.as_str()),
                ) {
                    (Ok(c), Ok(y), Ok(g), Ok(p)) => Ok((c, y, g, p)),
                    _ => sled::transaction::abort(DbTxErr::new("")),
                }?;

                let verify = Verify::new(&choice, &res, &c, &y, &g, &p, data.tests.len() + 1);

                client_test.valid = Some(verify.valid);

                data.tests.push(client_test.clone());
                tx_db
                    .insert(
                        client_id.clone(),
                        serde_json::to_string(&data).unwrap().as_str(),
                    )
                    .unwrap();

                Ok(verify)
            });

            Ok(HttpResponse::Ok().json(&tx_res.unwrap()))
        }
    }
}

#[derive(Debug, Serialize)]
struct Authenticated {
    auth: bool,
}

#[web::get("/authenticated")]
async fn authenticated(req: web::HttpRequest) -> Result<HttpResponse, web::Error> {
    match req.headers().get("x-client-id").unwrap().to_str() {
        Ok("") | Err(_) => {
            let err = JsonErr::new("no client id present");

            Ok(HttpResponse::BadRequest().json(&err))
        }
        Ok(client_id) => {
            let tx_res: TransactionResult<Authenticated, DbTxErr> = DB.transaction(|tx_db| {
                let mut data: ClientData =
                    serde_json::from_slice(tx_db.get(client_id).unwrap().unwrap().borrow())
                        .unwrap();

                let auth = data
                    .tests
                    .iter()
                    .all(|test| test.valid.map_or(false, |v| v));

                data.auth = Some(auth);
                data.tests = vec![];

                tx_db
                    .insert(
                        client_id.clone(),
                        serde_json::to_string(&data).unwrap().as_str(),
                    )
                    .unwrap();

                Ok(Authenticated { auth })
            });

            Ok(HttpResponse::Ok().json(&tx_res.unwrap()))
        }
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    web::server(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3080")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .max_age(3600)
                    .finish(),
            )
            .app_state(web::types::JsonConfig::default().limit(1024))
            .service((
                healthcheck,
                handshake,
                public_key,
                pick_choice,
                verify,
                authenticated,
            ))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_traits::Num;
    use openssl::bn::{BigNum, MsbOption};
    use pwd_dl_zkp_peggy::peggy::Peggy;
    use pwd_dl_zkp_victor::victor::Victor;
    use rand::thread_rng;

    fn bigint_to_bignum(i: &BigInt) -> BigNum {
        BigNum::from_hex_str(i.to_str_radix(16).as_str()).unwrap()
    }

    fn bignum_to_bigint(i: &BigNum) -> BigInt {
        BigInt::from_str_radix(i.to_hex_str().unwrap().to_string().as_str(), 16).unwrap()
    }

    #[test]
    fn protocol() {
        let mut rng = thread_rng();

        let mut x_bignum = BigNum::new().unwrap();
        x_bignum.rand(32, MsbOption::MAYBE_ZERO, false).unwrap();
        let x_bigint = bignum_to_bigint(&x_bignum);

        let victor = Victor::new();

        let (p_bignum, g_bignum) = victor.handshake().unwrap();
        let p_bigint = bignum_to_bigint(&p_bignum);
        let g_bigint = bignum_to_bigint(&g_bignum);

        let y_bigint = Peggy::public_key(&x_bigint, &g_bigint, &p_bigint).unwrap();
        let y_bignum = bigint_to_bignum(&y_bigint);

        let check = (0..1000)
            .into_iter()
            .map(|_| {
                let r_bigint = Peggy::gen_r(&p_bigint).unwrap();

                let c_bigint = Peggy::calc_c(&r_bigint, &g_bigint, &p_bigint).unwrap();
                let c_bignum = bigint_to_bignum(&c_bigint);

                let choice = victor.pick_choice(&mut rng);

                let res_bigint =
                    Peggy::calc_choice(&choice, &x_bigint, &r_bigint, &p_bigint).unwrap();
                let res_bignum = bigint_to_bignum(&res_bigint);

                victor.verify(
                    &choice,
                    &res_bignum,
                    &c_bignum,
                    &y_bignum,
                    &g_bignum,
                    &p_bignum,
                )
            })
            .all(|res| res.is_ok() && res.unwrap());

        assert_eq!(check, true);
    }
}
