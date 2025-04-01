use super::{Cip30Api, Extension};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_namespace = ["window"], js_name = "cardano")]
    pub static WALLETS: js_sys::Object;
    #[wasm_bindgen(thread_local_v2, js_namespace = ["window", "cardano"], js_name = "lace")]
    pub static LACE: Option<Cip30Wallet>;
    #[wasm_bindgen(thread_local_v2, js_namespace = ["window", "cardano"], js_name = "flint")]
    pub static FLINT: Option<Cip30Wallet>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, PartialEq)]
    pub type Cip30Wallet;

    /// A name for the wallet which can be used inside of the dApp for the
    /// purpose of asking the user which wallet they would like to connect with.
    #[wasm_bindgen(method, getter)]
    pub fn name(this: &Cip30Wallet) -> String;

    /// The version number of the API that the wallet supports.
    #[wasm_bindgen(method, getter, js_name = "apiVersion")]
    pub fn version(this: &Cip30Wallet) -> String;

    /// A URI image (e.g. data URI base64 or other) for img src for the wallet
    /// which can be used inside of the dApp for the purpose of asking the user
    /// which wallet they would like to connect with.
    #[wasm_bindgen(method, getter)]
    pub fn icon(this: &Cip30Wallet) -> String;

    /// Returns available wallet extensions that dApps can request. Note: requesting conflicting
    /// extensions may result in some being disabled. Check api.getExtensions() after initialisation.
    #[wasm_bindgen(method, getter)]
    pub fn supported_extensions(this: &Cip30Wallet) -> Vec<Extension>;

    /// Check if the dApp is connected to the wallet. Returns true if connected
    /// or whitelisted, indicating wallet.enable() will succeed without prompts.
    #[wasm_bindgen(method, catch, js_name = "isEnabled")]
    pub async fn enabled(this: &Cip30Wallet) -> Result<JsValue, JsValue>;

    /// Establishes initial connection with user's wallet, returning a full API
    /// object. Prompts for user permission on first connect, subsequent
    /// connections may use cached permissions.
    ///
    /// Usage:
    /// - Pass list of required CIP extension numbers to specify needed
    ///   functionality
    /// - Wallets support CIP-0030 base interface plus optional extensions
    /// - Extensions may conflict - wallets will enable what they can support
    /// - Check api.getExtensions() to verify available functionality
    /// - Extension endpoints are namespaced as .cipXXXX (no leading zeros)
    ///
    /// More details [CIP-0030](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0030#cardanowalletnameenable-extensions-extension----promiseapi)
    ///
    #[wasm_bindgen(method, catch, js_name = "enable")]
    pub async fn enable(this: &Cip30Wallet, extensions: JsValue) -> Result<Cip30Api, JsValue>;
}
