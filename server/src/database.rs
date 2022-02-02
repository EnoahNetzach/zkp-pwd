use once_cell::sync::Lazy;
use std::sync::Mutex;

pub(crate) static DB: Lazy<Mutex<sled::Db>> =
    Lazy::new(|| Mutex::new(sled::open("db/client_data").unwrap()));
