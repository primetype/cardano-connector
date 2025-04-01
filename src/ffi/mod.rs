pub mod cip30;
pub mod cip30_api;

pub use self::{cip30::Cip30Wallet, cip30_api::Cip30Api};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct Extension {
    pub cip: u64,
}
