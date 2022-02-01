pub mod core {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum Choice {
        XRMP,
        R,
    }
}
