pub mod peggy {
    use num_bigint::{BigInt, ParseBigIntError, RandBigInt, ToBigInt};
    use pwd_dl_zkp_core::core::Choice;

    pub struct Peggy {}

    impl Peggy {
        pub fn public_key(x: &BigInt, g: &BigInt, p: &BigInt) -> Result<BigInt, ParseBigIntError> {
            let y = g.modpow(&x, &p);

            Ok(y)
        }

        pub fn gen_r(p: &BigInt) -> Result<BigInt, ParseBigIntError> {
            let mut rng = rand::thread_rng();
            let low = 0i32.to_bigint().unwrap();
            let high = p - 2;
            let r = rng.gen_bigint_range(&low, &high);

            Ok(r)
        }

        pub fn calc_c(r: &BigInt, g: &BigInt, p: &BigInt) -> Result<BigInt, ParseBigIntError> {
            let c = g.modpow(&r, &p);

            Ok(c)
        }

        pub fn calc_choice(
            choice: &Choice,
            x: &BigInt,
            r: &BigInt,
            p: &BigInt,
        ) -> Result<BigInt, ParseBigIntError> {
            match choice {
                Choice::R => Ok(r.clone()),
                Choice::XRMP => Ok((x + r) % (p - 1)),
            }
        }
    }
}
