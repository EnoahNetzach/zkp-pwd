pub mod victor {
    use openssl::bn::{BigNum, BigNumContext};
    use openssl::error::ErrorStack;
    use pwd_dl_zkp_core::core::Choice;
    use rand::rngs::ThreadRng;
    use rand::Rng;

    const BITS: i32 = 512;

    fn generate_safe_prime() -> Result<BigNum, ErrorStack> {
        let mut prime = BigNum::new()?;
        prime.generate_prime(BITS, true, None, None)?;

        Ok(prime)
    }

    fn find_cyclic_group_generator(p: &BigNum) -> Result<BigNum, ErrorStack> {
        let mut q = BigNum::from_dec_str(p.to_dec_str()?.to_string().as_str())?;
        q.sub_word(1)?;
        q.div_word(2)?;

        let one = BigNum::from_u32(1)?;
        let two = BigNum::from_u32(2)?;
        let mut bnctx = BigNumContext::new()?;

        let mut alpha = BigNum::new()?;
        let mut b = BigNum::new()?;

        loop {
            p.rand_range(&mut alpha)?;

            if alpha <= one {
                continue;
            }

            b.clear();

            b.mod_exp(&alpha, &two, &p, &mut bnctx)?;

            if &b % p == one {
                continue;
            }

            b.mod_exp(&alpha, &q, &p, &mut bnctx)?;

            if &b % p == one {
                continue;
            }

            return Ok(alpha);
        }
    }

    #[derive(Clone)]
    pub struct Victor {}

    impl Victor {
        pub fn new() -> Self {
            Victor {}
        }

        pub fn handshake(&self) -> Result<(BigNum, BigNum), ErrorStack> {
            let p = generate_safe_prime()?;
            let g = find_cyclic_group_generator(&p)?;

            return Ok((p, g));
        }

        pub fn pick_choice(&self, rng: &mut ThreadRng) -> Choice {
            if rng.gen::<bool>() {
                Choice::XRMP
            } else {
                Choice::R
            }
        }

        pub fn verify(
            &self,
            choice: &Choice,
            res: &BigNum,
            c: &BigNum,
            y: &BigNum,
            g: &BigNum,
            p: &BigNum,
        ) -> Result<bool, ErrorStack> {
            let mut bnctx = BigNumContext::new()?;

            let mut lhs = BigNum::new()?;
            let mut rhs = BigNum::new()?;
            rhs.mod_exp(&g, &res, &p, &mut bnctx)?;

            match choice {
                Choice::R => Ok(c == &rhs),
                Choice::XRMP => {
                    lhs.mod_mul(&c, &y, &p, &mut bnctx)?;

                    Ok(lhs == rhs)
                }
            }
        }
    }
}
