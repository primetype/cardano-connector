use crate::{
    Address, Wallet,
    cardano::{Coin, Hash, TransactionBody, Tx, Utxo, Value, WitnessSet},
    error::{APIError, APIErrorCode, PaginateError},
    ffi::{
        self,
        cip30_api::{self, DataSignature, Paginate},
    },
};
use core::fmt;

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

#[derive(Clone, PartialEq)]
pub struct ConnectedWallet {
    wallet: Wallet,
    cip30_api: cip30_api::Cip30Api,
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
                    let address = Address::from_hex(&address).map_err(|err| APIError {
                        code: APIErrorCode::InternalError,
                        info: err.to_string(),
                    })?;
                    unused_addresses.push(address);
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
                    let address = Address::from_hex(&address).map_err(|err| APIError {
                        code: APIErrorCode::InternalError,
                        info: err.to_string(),
                    })?;
                    unused_addresses.push(address);
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
                let address = Address::from_hex(&address).map_err(|err| APIError {
                    code: APIErrorCode::InternalError,
                    info: err.to_string(),
                })?;
                Ok(address)
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
                    let address = Address::from_hex(&address).map_err(|err| APIError {
                        code: APIErrorCode::InternalError,
                        info: err.to_string(),
                    })?;
                    unused_addresses.push(address);
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

    pub async fn sign_data(
        &self,
        address: &Address,
        payload: impl AsRef<[u8]>,
    ) -> Result<SignedData, APIError> {
        // encode the payload in hexadecimal as required by the CIP-30 api
        let address = address.to_hex();
        let payload = hex::encode(payload);

        // sign the payload using the connected wallet
        match self.cip30_api.sign_data(&address, &payload).await {
            Ok(signature) => SignedData::try_from(signature),
            Err(error) => {
                // TODO: handle signature error

                serde_wasm_bindgen::from_value(error.clone())
                    .map_err(|decode_error| APIError {
                        code: APIErrorCode::InternalError,
                        info: format!(
                            "Couldn't decode the error content: {decode_error} ({error:?})"
                        ),
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

pub struct SignedData {
    pub key: [u8; 32],
    pub signature: [u8; 64],
    pub signed_data: Vec<u8>,
    pub address: Vec<u8>,
}

fn cbor_to_api(error: cbor_event::Error) -> APIError {
    APIError {
        code: APIErrorCode::Unknown(42),
        info: format!("{error}"),
    }
}

fn extract_address_from_protected_header(bytes: &[u8]) -> Result<Vec<u8>, APIError> {
    use cbor_event::{Deserialize as _, Len, Value, de::Deserializer};

    let mut cbor = Deserializer::from(bytes);

    let len = cbor.map().map_err(cbor_to_api)?;
    let mut read = 0;

    while match len {
        Len::Len(n) => read < n,
        Len::Indefinite => true,
    } {
        let key = Value::deserialize(&mut cbor).map_err(cbor_to_api)?;

        if key == Value::Special(cbor_event::Special::Break) {
            break;
        } else if let Value::Text(key) = key {
            if key == "address" {
                // don't dwell, we can already return and stop there
                return cbor.bytes().map_err(cbor_to_api);
            }
        } else {
            // ignore the value and move to the next entry
            let _value = Value::deserialize(&mut cbor).map_err(cbor_to_api)?;
        }

        read += 1;
    }

    Err(APIError {
        code: APIErrorCode::Unknown(42),
        info: "Invalid cbor, missing address".to_owned(),
    })
}

fn extract_cose_key(bytes: &[u8]) -> Result<[u8; 32], APIError> {
    use cbor_event::{Deserialize as _, Len, Value, de::Deserializer};

    let mut cbor = Deserializer::from(bytes);

    let len = cbor.map().map_err(cbor_to_api)?;
    let mut read = 0;

    while match len {
        Len::Len(n) => read < n,
        Len::Indefinite => true,
    } {
        let key = Value::deserialize(&mut cbor).map_err(cbor_to_api)?;

        if key == Value::Special(cbor_event::Special::Break) {
            break;
        } else if Value::I64(-2) == key {
            // don't dwell, we can already return and stop there
            let mut key = [0; 32];
            let key_bytes = cbor.bytes().map_err(cbor_to_api)?;
            assert_eq!(key_bytes.len(), 32);
            key.copy_from_slice(&key_bytes);
            return Ok(key);
        } else {
            // ignore the value and move to the next entry
            let _value = Value::deserialize(&mut cbor).map_err(cbor_to_api)?;
        }

        read += 1;
    }

    Err(APIError {
        code: APIErrorCode::Unknown(42),
        info: "Invalid cbor, missing key".to_owned(),
    })
}

fn decode_cose_sig1(bytes: &[u8]) -> Result<SignedData, APIError> {
    use cbor_event::{Deserialize as _, Len, Value, de::Deserializer, se::Serializer};

    let mut cbor = Deserializer::from(bytes);

    let _len = cbor.array().map_err(cbor_to_api)?;

    let protected_header = cbor.bytes().map_err(cbor_to_api)?;
    let address = extract_address_from_protected_header(&protected_header)?;

    // unprotected
    let () = cbor
        .map_with(|cbor| {
            let _key = Value::deserialize(cbor)?;
            let _value = Value::deserialize(cbor)?;
            Ok(())
        })
        .map_err(cbor_to_api)?;

    let data = cbor.bytes().map_err(cbor_to_api)?;

    let signature_bytes = cbor.bytes().map_err(cbor_to_api)?;
    assert_eq!(signature_bytes.len(), 64);
    let mut signature = [0; 64];
    signature.copy_from_slice(&signature_bytes);

    let mut signed_data = Serializer::new_vec();

    signed_data.write_array(Len::Len(4)).map_err(cbor_to_api)?;
    signed_data.write_text("Signature1").map_err(cbor_to_api)?;
    signed_data
        .write_bytes(&protected_header)
        .map_err(cbor_to_api)?;
    signed_data.write_bytes([]).map_err(cbor_to_api)?; // external aad empty
    signed_data.write_bytes(data).map_err(cbor_to_api)?;

    let signed_data = signed_data.finalize();

    Ok(SignedData {
        key: [0; 32],
        signature,
        signed_data,
        address,
    })
}

impl SignedData {
    fn try_from(signature: DataSignature) -> Result<Self, APIError> {
        Self::from_bytes(&signature.key(), &signature.signature())
    }

    fn from_bytes(key_bytes: &str, signature_bytes: &str) -> Result<Self, APIError> {
        let signature = hex::decode(signature_bytes).map_err(|decode_error| APIError {
            code: APIErrorCode::InternalError,
            info: format!("Couldn't decode the signature bytes: {decode_error}"),
        })?;
        let key = hex::decode(key_bytes).map_err(|decode_error| APIError {
            code: APIErrorCode::InternalError,
            info: format!("Couldn't decode the key bytes: {decode_error}"),
        })?;

        let key = extract_cose_key(&key)?;
        Ok(Self {
            key,
            ..decode_cose_sig1(&signature)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COSE_KEY: &str = "a50101025839012abad624652d7b0fa0d00744a767b09dd0c9e8ef890966103802be7875d8bdc2a0facf8b85d2457c2417098703cf29067cea3eec03071f6e0327200621582074647c101ed98ade960ebad955f60961d1fcf77cb8a0bac9d6b778227685d1ae";
    const COSE_SIG: &str = "845882a30127045839012abad624652d7b0fa0d00744a767b09dd0c9e8ef890966103802be7875d8bdc2a0facf8b85d2457c2417098703cf29067cea3eec03071f6e67616464726573735839012abad624652d7b0fa0d00744a767b09dd0c9e8ef890966103802be7875d8bdc2a0facf8b85d2457c2417098703cf29067cea3eec03071f6ea166686173686564f4446461746158402b45771561fdb6041326331a101a99d4bfe4f1a5c5b007f3d2f4f2e7f3f34d45aa5fedcd3f520e1799974c707996475693170531e2ad4a05ece3beb456f35a0f";

    #[test]
    fn signed_data_from_bytes() {
        let result = SignedData::from_bytes(COSE_KEY, COSE_SIG).unwrap();

        dbg!(hex::encode(&result.key));
        dbg!(hex::encode(&result.signature));
        dbg!(hex::encode(&result.signed_data));

        assert!(cryptoxide::ed25519::verify(
            &result.signed_data,
            &result.key,
            &result.signature
        ));
    }
}
