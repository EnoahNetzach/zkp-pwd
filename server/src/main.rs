use crate::router::routes;
use ntex::web;
use ntex::web::{middleware, App};
use ntex_cors::Cors;

mod client_data;
mod database;
mod router;
mod web_utils;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    web::server(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3080")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["content-type", "x-client-id"])
                    .max_age(3600)
                    .finish(),
            )
            .app_state(web::types::JsonConfig::default().limit(1024))
            .service(routes())
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
