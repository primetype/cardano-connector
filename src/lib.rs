/*!

# Cardano Connector for CIP30 wallet

This library is meant to be used for web applications that need to interact with Cardano wallets.
It provides a simple and easy-to-use interface for managing Cardano wallets and their associated data.

## Features

- Connect to Cardano wallets
- Manage wallet addresses
- Retrieve wallet balance
- Send transactions

## Usage

First list all the wallets available:

```no_run
use cardano_connector::wallets;

for wallet in wallets() {
    println!("Wallet: {} ({})", wallet.name(), wallet.version());
}
```

Only the wallets that are implementing the CIP30 standard and are enabled in the browser
will be listed. The [`wallets`] function returns a vector of [`Wallet`] instances.

This will gives you limited information about the wallet application. To access the wallet's
addresses, balance and create and send transactions, you need to enable the wallet, which
will return a [`ConnectedWallet`] instance.

```no_run
# use cardano_connector::wallets;
#
# async fn test() -> anyhow::Result<()> {
# let wallet = wallets().pop().unwrap();
let connected_wallet = wallet.enable().await?;
# Ok(()) }
```

*/

pub mod cardano;
mod connected_wallet;
pub mod error;
pub mod ffi;
mod wallet;

pub use self::{
    cardano::Utxo,
    connected_wallet::{Address, ConnectedWallet, NetworkId},
    wallet::{Wallet, wallets},
};
