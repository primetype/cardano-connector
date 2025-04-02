use core::fmt;

use pallas_primitives::Bytes;

use crate::{
    Wallet,
    cardano::{Coin, Hash, TransactionBody, Tx, Utxo, Value, WitnessSet},
    error::{APIError, APIErrorCode, PaginateError},
    ffi::{
        self,
        cip30_api::{self, Paginate},
    },
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum NetworkId {
    PreProduction,
    Preview,
    Mainnet,
    Unknown(u8),
}
impl From<NetworkId> for u8 {
    fn from(network_id: NetworkId) -> Self {
        match network_id {
            NetworkId::PreProduction => 0,
            NetworkId::Preview => 0,
            NetworkId::Mainnet => 1,
            NetworkId::Unknown(n) => n,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(String);

#[derive(Clone, PartialEq)]
pub struct ConnectedWallet {
    wallet: Wallet,
    cip30_api: cip30_api::Cip30Api,
}

impl Address {
    pub fn to_bytes(&self) -> Bytes {
        hex::decode(&self.0).unwrap().into()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <String as fmt::Display>::fmt(&self.0, f)
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkId::PreProduction => write!(f, "pre-production"),
            NetworkId::Preview => write!(f, "preview"),
            NetworkId::Mainnet => write!(f, "mainnet"),
            NetworkId::Unknown(id) => write!(f, "unknown-network-id({id:#02x})"),
        }
    }
}

impl ConnectedWallet {
    pub(crate) fn new(wallet: Wallet, cip30_api: cip30_api::Cip30Api) -> Self {
        Self { wallet, cip30_api }
    }

    /// return the name of the wallet connector application
    pub fn name(&self) -> String {
        self.wallet.name()
    }

    /// return the wallet connector application's version
    pub fn version(&self) -> String {
        self.wallet.version()
    }

    /// returns the HTML ready Icon for this wallet connector application
    pub fn icon(&self) -> String {
        self.wallet.icon()
    }

    /// list the supported extensions of this wallet connector application
    pub fn supported_extensions(&self) -> Vec<ffi::Extension> {
        self.wallet.supported_extensions()
    }

    /// list the enabled extensions with this wallet connector
    pub async fn enabled_extensions(&self) -> Result<Vec<ffi::Extension>, APIError> {
        match self.cip30_api.get_extensions().await {
            Ok(array) => serde_wasm_bindgen::from_value(array).map_err(|decode_array| APIError {
                code: APIErrorCode::InternalError,
                info: format!("Couldn't decode the extension list: {decode_array}"),
            }),
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// re-enable the connection to the wallet
    ///
    /// This is particularly useful is we received an [`APIErrorCode::AccountChange`]
    ///
    /// [`APIErrorCode::AccountChange`]: crate::error::APIErrorCode::AccountChange
    ///
    pub async fn enable(&mut self) -> Result<(), APIError> {
        self.cip30_api = self.wallet.enable().await?.cip30_api;
        Ok(())
    }

    /// returns the network identifier. It allows us to at least detect if we are
    /// using a testing environment or a production environment. We won't have
    /// much information about which specific network identifier we are connected
    /// to.
    pub async fn network_id(&self) -> Result<NetworkId, APIError> {
        match self.cip30_api.network_id().await {
            Ok(id) => {
                if let Some(number) = id.as_f64() {
                    match number as u8 {
                        0 => Ok(NetworkId::PreProduction),
                        1 => Ok(NetworkId::Mainnet),
                        unknown => Ok(NetworkId::Unknown(unknown)),
                    }
                } else {
                    Err(APIError {
                        code: APIErrorCode::InternalError,
                        info: format!("Unknown network id: {id:?}"),
                    })
                }
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// get the [`Coin`] balance of this wallet
    pub async fn balance(&self) -> Result<Coin, APIError> {
        match self.cip30_api.balance().await {
            Ok(balance) => {
                let Some(balance_hex) = balance.as_string() else {
                    return Err(APIError {
                        code: APIErrorCode::InternalError,
                        info: format!("Unknown balance: {balance:?}"),
                    });
                };

                let balance_cbor = hex::decode(&balance_hex).map_err(|error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Invalid balance `{balance_hex:?}': {error}"),
                })?;

                pallas_codec::minicbor::decode(&balance_cbor).map_err(|error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Invalid balance `{balance_cbor:?}': {error}"),
                })
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// list all the used address of this connected wallet
    pub async fn used_addresses(
        &self,
        pagination: Option<Paginate>,
    ) -> Result<Vec<Address>, APIError> {
        match self.cip30_api.get_used_addresses(pagination).await {
            Ok(addresses) => {
                let mut unused_addresses = Vec::with_capacity(addresses.length() as usize);
                for address in addresses {
                    let Some(address) = address.as_string() else {
                        return Err(APIError {
                            code: APIErrorCode::InternalError,
                            info: format!("Invalid address: {address:?}"),
                        });
                    };
                    unused_addresses.push(Address(address));
                }
                Ok(unused_addresses)
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// list the unused addresses of the connected wallet
    pub async fn unused_addresses(&self) -> Result<Vec<Address>, APIError> {
        match self.cip30_api.get_unused_addresses().await {
            Ok(addresses) => {
                let mut unused_addresses = Vec::with_capacity(addresses.length() as usize);
                for address in addresses {
                    let Some(address) = address.as_string() else {
                        return Err(APIError {
                            code: APIErrorCode::InternalError,
                            info: format!("Invalid address: {address:?}"),
                        });
                    };
                    unused_addresses.push(Address(address));
                }
                Ok(unused_addresses)
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// list the change address
    pub async fn change_address(&self) -> Result<Address, APIError> {
        match self.cip30_api.get_change_address().await {
            Ok(address) => {
                let Some(address) = address.as_string() else {
                    return Err(APIError {
                        code: APIErrorCode::InternalError,
                        info: format!("Invalid address: {address:?}"),
                    });
                };

                Ok(Address(address))
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// list the reward address
    pub async fn reward_addresses(&self) -> Result<Vec<Address>, APIError> {
        match self.cip30_api.reward_addresses().await {
            Ok(addresses) => {
                let mut unused_addresses = Vec::with_capacity(addresses.length() as usize);
                for address in addresses {
                    let Some(address) = address.as_string() else {
                        return Err(APIError {
                            code: APIErrorCode::InternalError,
                            info: format!("Invalid address: {address:?}"),
                        });
                    };
                    unused_addresses.push(Address(address));
                }
                Ok(unused_addresses)
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// select Unspent transaction output that sumup to the given value
    pub async fn select_utxos(
        &self,
        value: &Value,
        pagination: Option<Paginate>,
    ) -> Result<Vec<Utxo>, APIError> {
        self._utxos(Some(value), pagination).await
    }

    /// returns all the UTxO without trying to sum up to a given value
    pub async fn all_utxos(&self, pagination: Option<Paginate>) -> Result<Vec<Utxo>, APIError> {
        self._utxos(None, pagination).await
    }

    async fn _utxos(
        &self,
        value: Option<&Value>,
        pagination: Option<Paginate>,
    ) -> Result<Vec<Utxo>, APIError> {
        let value = if let Some(value) = value {
            let bytes = pallas_codec::minicbor::to_vec(value).map_err(|error| APIError {
                code: APIErrorCode::InternalError,
                info: format!("Failed to encode value in cbor: {error}"),
            })?;
            Some(hex::encode(bytes))
        } else {
            None
        };

        match self.cip30_api.get_utxos(value, pagination).await {
            Ok(cbored_utxos) => {
                if cbored_utxos.is_null() {
                    return Ok(Vec::new());
                }

                let mut utxos = Vec::new();

                for element in cbored_utxos {
                    let hex = hex::decode(element.as_string().unwrap()).unwrap();
                    let utxo: Utxo = pallas_codec::minicbor::decode(&hex).unwrap();
                    utxos.push(utxo);
                }

                Ok(utxos)
            }
            Err(error) => {
                if let Ok(PaginateError { .. }) = serde_wasm_bindgen::from_value(error.clone()) {
                    return Ok(Vec::new());
                }

                serde_wasm_bindgen::from_value(error)
                    .map_err(|decode_error| APIError {
                        code: APIErrorCode::InternalError,
                        info: format!("Couldn't decode the error content: {decode_error}"),
                    })
                    .and_then(Err)
            }
        }
    }

    /// sign the given transaction
    pub async fn sign_tx(
        &self,
        transaction: &TransactionBody,
        partial_sign: bool,
    ) -> Result<WitnessSet, APIError> {
        let transaction_cbor = pallas_codec::minicbor::to_vec(transaction).unwrap();
        let transaction_hex = hex::encode(transaction_cbor);
        match self.cip30_api.sign_tx(&transaction_hex, partial_sign).await {
            Ok(set_js) => {
                let set_hex = set_js.as_string().unwrap();
                let set_cbor = hex::decode(set_hex).map_err(|error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the witness set: {error}"),
                })?;
                pallas_codec::minicbor::decode(&set_cbor).map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the witness set: {decode_error}"),
                })
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// ask the wallet connector application to submit the given transaction
    pub async fn submit_tx(&self, transaction: &Tx) -> Result<Hash<32>, APIError> {
        let transaction_cbor = pallas_codec::minicbor::to_vec(transaction).unwrap();
        let transaction_hex = hex::encode(transaction_cbor);
        match self.cip30_api.submit_tx(&transaction_hex).await {
            Ok(tx_hash_js) => {
                // TODO
                panic!("Don't know yet what is the output of submit: {tx_hash_js:?}");
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }
}
