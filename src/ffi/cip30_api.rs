use js_sys::{Array, JsString};
use wasm_bindgen::prelude::*;

/// Used to specify optional pagination for some API calls. Limits results to
/// `limit` each page, and uses a 0-indexing `page` to refer to which of those
/// pages of `limit` items each. dApps should be aware that if a wallet is
/// modified between paginated calls that this will change the pagination, e.g.
/// some results skipped or showing up multiple times but otherwise the wallet
/// must respect the pagination order.
#[wasm_bindgen]
pub struct Paginate {
    /// the page index
    pub page: usize,
    /// the limit of elements per pages
    pub limite: usize,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum NetworkId {
    /// Pre-Production or Preview
    Testing = 0,
    #[default]
    Mainnet = 1,
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, PartialEq)]
    pub type DataSignature;

    #[wasm_bindgen(method, getter, js_name = "signature")]
    pub fn signature(this: &DataSignature) -> String;
    #[wasm_bindgen(method, getter, js_name = "key")]
    pub fn key(this: &DataSignature) -> String;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, PartialEq)]
    pub type Cip30Api;

    #[wasm_bindgen(method, catch, js_name = "getExtensions")]
    pub async fn get_extensions(this: &Cip30Api) -> Result<JsValue, JsValue>;

    /// Returns the network id of the currently connected account.
    /// 0 is testnet and 1 is mainnet but other networks can possibly be
    /// returned by wallets. Those other network ID values are not
    /// governed by this document. This result will stay the same unless
    /// the connected account has changed
    #[wasm_bindgen(method, catch, js_name = "getNetworkId")]
    pub async fn network_id(this: &Cip30Api) -> Result<JsValue, JsValue>;

    /// If amount is undefined, this shall return a list of all UTXOs (unspent
    /// transaction outputs) controlled by the wallet. If amount is not
    /// undefined, this request shall be limited to just the UTXOs that are
    /// required to reach the combined ADA/multiasset value target specified
    /// in amount, and if this cannot be attained, null shall be returned. The
    /// results can be further paginated by paginate if it is not undefined.
    #[wasm_bindgen(method, catch, js_name = "getUtxos")]
    pub async fn get_utxos(
        this: &Cip30Api,
        amount: Option<String>,
        pagination: Option<Paginate>,
    ) -> Result<Array, JsValue>;
    /// Returns an address owned by the wallet that should be used as a change
    /// address to return leftover assets during transaction creation back to
    /// the connected wallet. This can be used as a generic receive address as
    /// well.
    ///
    /// The address is encoded as hexadecimal string.
    #[wasm_bindgen(method, catch, js_name = "getChangeAddress")]
    pub async fn get_change_address(this: &Cip30Api) -> Result<JsString, JsValue>;
    /// Returns the total balance available of the wallet. This is the same as
    /// summing the results of api.getUtxos(), but it is both useful to dApps
    /// and likely already maintained by the implementing wallet in a more
    /// efficient manner so it has been included in the API as well.
    ///
    /// Hexadecimal string of the cbor encoded Number
    #[wasm_bindgen(method, catch, js_name = "getBalance")]
    pub async fn balance(this: &Cip30Api) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch, js_name = "getUsedAddresses")]
    pub async fn get_used_addresses(
        this: &Cip30Api,
        paginate: Option<Paginate>,
    ) -> Result<Array, JsValue>;
    #[wasm_bindgen(method, catch, js_name = "getUnusedAddresses")]
    pub async fn get_unused_addresses(this: &Cip30Api) -> Result<Array, JsValue>;
    /// Returns the reward addresses owned by the wallet. This can return
    /// multiple addresses e.g. CIP-0018.
    #[wasm_bindgen(method, catch, js_name = "getRewardAddresses")]
    pub async fn reward_addresses(this: &Cip30Api) -> Result<Array, JsValue>;

    /// Requests that a user sign the unsigned portions of the supplied
    /// transaction. The wallet should ask the user for permission, and if
    /// given, try to sign the supplied body and return a signed transaction.
    ///
    /// If `partial_sign` is `true`, the wallet only tries to sign what it can.
    /// If `partial_sign` is false and the wallet could not sign the entire
    /// transaction TxSignError shall be returned with the ProofGeneration
    /// code. Likewise if the user declined in either case it shall return the
    /// UserDeclined code.
    ///
    /// Only the portions of the witness set that were
    /// signed as a result of this call are returned to encourage dApps to
    /// verify the contents returned by this endpoint while building the final
    /// transaction.
    ///
    /// Returns the hexadecimal encoded cbor of the transaction witness set
    #[wasm_bindgen(method, catch, js_name = "signTx")]
    pub async fn sign_tx(
        this: &Cip30Api,
        transaction: &str,
        partial_sign: bool,
    ) -> Result<JsString, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "signData")]
    pub async fn sign_data(
        this: &Cip30Api,
        addr: &str,
        data: &str,
    ) -> Result<DataSignature, JsValue>;

    /// As wallets should already have this ability, we allow dApps to request
    /// that a transaction be sent through it. If the wallet accepts the
    /// transaction and tries to send it, it shall return the transaction id
    /// for the dApp to track. The wallet is free to return the TxSendError
    /// with code Refused if they do not wish to send it, or Failure if there
    /// was an error in sending it (e.g. preliminary checks failed on
    /// signatures).
    #[wasm_bindgen(method, catch, js_name = "submitTx")]
    pub async fn submit_tx(this: &Cip30Api, transaction: &str) -> Result<JsString, JsValue>;

}
