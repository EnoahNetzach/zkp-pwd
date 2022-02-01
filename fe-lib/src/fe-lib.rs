use num_bigint::BigInt;
use num_traits::Num;
use pwd_dl_zkp_core::core::Choice;
use pwd_dl_zkp_peggy::peggy::Peggy;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn public_key(x: &str, g: &str, p: &str) -> Result<String, JsError> {
    let x = BigInt::from_str_radix(x, 16)?;
    let g = BigInt::from_str_radix(g, 16)?;
    let p = BigInt::from_str_radix(p, 16)?;

    let y = Peggy::public_key(&x, &g, &p)?;

    Ok(y.to_str_radix(16))
}

#[wasm_bindgen]
pub fn gen_r(p: &str) -> Result<String, JsError> {
    let p = BigInt::from_str_radix(p, 16)?;

    let r = Peggy::gen_r(&p)?;

    Ok(r.to_str_radix(16))
}

#[wasm_bindgen]
pub fn calc_c(r: &str, g: &str, p: &str) -> Result<String, JsError> {
    let r = BigInt::from_str_radix(r, 16)?;
    let g = BigInt::from_str_radix(g, 16)?;
    let p = BigInt::from_str_radix(p, 16)?;

    let c = Peggy::calc_c(&r, &g, &p)?;

    Ok(c.to_str_radix(16))
}

#[wasm_bindgen]
pub fn calc_choice(choice: &JsValue, x: &str, r: &str, p: &str) -> Result<String, JsError> {
    let choice: Choice = choice.into_serde()?;
    let x = BigInt::from_str_radix(x, 16)?;
    let r = BigInt::from_str_radix(r, 16)?;
    let p = BigInt::from_str_radix(p, 16)?;

    let res = Peggy::calc_choice(&choice, &x, &r, &p)?;

    Ok(res.to_str_radix(16))
}
