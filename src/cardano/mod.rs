#[cfg(feature = "transaction")]
use crate::Address;
use pallas_codec::minicbor;
#[cfg(feature = "transaction")]
use pallas_primitives::babbage::PseudoPostAlonzoTransactionOutput;
use pallas_primitives::conway::PseudoTransactionOutput;
pub use pallas_primitives::{
    AssetName, Coin, Hash, NonEmptyKeyValuePairs, PolicyId, PositiveCoin, TransactionIndex,
    TransactionInput,
    alonzo::Value as LegacyValue,
    conway::{Multiasset, TransactionBody, TransactionOutput, Tx, Value, WitnessSet},
};
#[cfg(feature = "transaction")]
use pallas_txbuilder::{Input, StagingTransaction};
use std::collections::HashMap;
#[cfg(feature = "transaction")]
use thiserror::Error;

/// decode the CBOR encoded UTxO as returned from the CIP30 getUtxos
/// API.
#[derive(Debug, PartialEq, Eq, Clone, pallas_codec::minicbor::Decode)]
#[cbor(array)]
pub struct Utxo {
    #[n(0)]
    pub input: TransactionInput,
    #[n(1)]
    pub output: TransactionOutput,
}

impl Utxo {
    pub fn transaction_id(&self) -> Hash<32> {
        self.input.transaction_id
    }

    pub fn index(&self) -> u64 {
        self.input.index
    }

    pub fn amount(&self) -> Coin {
        match &self.output {
            TransactionOutput::Legacy(output) => match output.amount {
                LegacyValue::Coin(coin) => coin,
                LegacyValue::Multiasset(coin, _) => coin,
            },
            TransactionOutput::PostAlonzo(output) => match output.value {
                Value::Coin(coin) => coin,
                Value::Multiasset(coin, _) => coin,
            },
        }
    }

    pub fn address(&self) -> String {
        match &self.output {
            TransactionOutput::Legacy(output) => output.address.to_string(),
            TransactionOutput::PostAlonzo(output) => output.address.to_string(),
        }
    }
}

#[derive(Debug, Error)]
#[cfg(feature = "transaction")]
pub enum GroupUtxoError {
    #[error("Not enough to pay the fee ({fee}), available funds are {sum}.")]
    CantPayFee { fee: Coin, sum: Coin },
}

/// function to group the given list of UTxO into one output
///
/// TODO:
///
/// - [x] minus the fees
/// - [x] output address
/// - [ ] return the built transaction
#[cfg(feature = "transaction")]
pub fn group_utxos<'a>(
    utxos: impl IntoIterator<Item = &'a Utxo>,
    fee: Coin,
    to: Address,
) -> Result<TransactionOutput, GroupUtxoError> {
    // extract the network id from the received address and validate it against
    // the utxos outputs in the list
    let network_id = todo!();

    let mut inputs = Vec::new();
    let mut value = sumup(utxos.into_iter().map(|utxo| {
        inputs.push(utxo.input.clone());
        &utxo.output
    }));

    let staging = inputs
        .into_iter()
        .fold(StagingTransaction::new(), |staging, input| {
            staging.input(Input::new(input.transaction_id, input.index))
        });
    let staging = staging.network_id(network_id);

    // deduce the fees
    match &mut value {
        Value::Coin(c) | Value::Multiasset(c, _) => {
            let Some(rem) = c.checked_sub(fee) else {
                return Err(GroupUtxoError::CantPayFee { fee, sum: *c });
            };

            *c = rem;
        }
    }

    let address = to.to_bytes();
    let output = PseudoPostAlonzoTransactionOutput {
        address,
        value,
        datum_option: None,
        script_ref: None,
    };

    Ok(TransactionOutput::PostAlonzo(output))
}

pub fn sumup<'a>(outputs: impl IntoIterator<Item = &'a TransactionOutput>) -> Value {
    let mut coin = 0;
    let mut assets: HashMap<PolicyId, HashMap<AssetName, PositiveCoin>> = HashMap::new();

    for output in outputs {
        match output {
            PseudoTransactionOutput::Legacy(tx) => match &tx.amount {
                pallas_primitives::alonzo::Value::Coin(c) => {
                    coin += c;
                }
                pallas_primitives::alonzo::Value::Multiasset(c, multiasset) => {
                    coin += c;

                    for (cert, asset) in multiasset.iter() {
                        let entry = assets.entry(*cert).or_default();

                        for (asset_name, amount) in asset.iter() {
                            entry
                                .entry(asset_name.clone())
                                .and_modify(|t| {
                                    *t = PositiveCoin::try_from(u64::from(*t) + amount).unwrap()
                                })
                                .or_insert_with(|| PositiveCoin::try_from(*amount).unwrap());
                        }
                    }
                }
            },
            PseudoTransactionOutput::PostAlonzo(tx) => match &tx.value {
                Value::Coin(c) => {
                    coin += c;
                }
                Value::Multiasset(c, multiasset) => {
                    coin += c;

                    for (cert, asset) in multiasset.iter() {
                        let entry = assets.entry(*cert).or_default();

                        for (asset_name, amount) in asset.iter() {
                            entry
                                .entry(asset_name.clone())
                                .and_modify(|t| {
                                    *t = PositiveCoin::try_from(u64::from(*t) + u64::from(amount))
                                        .unwrap()
                                })
                                .or_insert_with(|| *amount);
                        }
                    }
                }
            },
        }
    }

    let assets = Multiasset::from_vec(
        assets
            .into_iter()
            .map(|(key, value)| (key, NonEmptyKeyValuePairs::Def(value.into_iter().collect())))
            .collect(),
    );

    if let Some(assets) = assets {
        Value::Multiasset(coin, assets)
    } else {
        Value::Coin(coin)
    }
}
