use std::path::{Path, PathBuf};

use namada_sdk::wallet::{LoadStoreError, Wallet};
use serde::Serialize;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;

use crate::cli::args;
use crate::config::genesis;
use crate::wallet::{pre_genesis, CliWalletUtils};


pub const NET_ACCOUNTS_DIR: &str = "setup";
pub const NET_OTHER_ACCOUNTS_DIR: &str = "other";
pub const ENV_VAR_NETWORK_CONFIGS_DIR: &str = "NAMADA_NETWORK_CONFIGS_DIR";
/// Github URL prefix of released Namada network configs
pub const ENV_VAR_NETWORK_CONFIGS_SERVER: &str =
    "NAMADA_NETWORK_CONFIGS_SERVER";
const DEFAULT_NETWORK_CONFIGS_SERVER: &str =
    "https://github.com/heliaxdev/anoma-network-config/releases/download";

/// We do pre-genesis validator set up in this directory
pub const PRE_GENESIS_DIR: &str = "pre-genesis";

/// Configure Namada to join an existing network. The chain must be released in
/// the <https://github.com/heliaxdev/anoma-network-config> repository.


/// Try to load a pre-genesis wallet or return nothing,
/// if it cannot be found.
pub fn try_load_pre_genesis_wallet(
    base_dir: &Path,
) -> Result<(Wallet<CliWalletUtils>, PathBuf), LoadStoreError> {
    let pre_genesis_dir = base_dir.join(PRE_GENESIS_DIR);

    crate::wallet::load(&pre_genesis_dir).map(|wallet| {
        let wallet_file = crate::wallet::wallet_file(&pre_genesis_dir);
        (wallet, wallet_file)
    })
}

/// Try to load a pre-genesis wallet or terminate if it cannot be found.
pub fn load_pre_genesis_wallet_or_exit(
    base_dir: &Path,
) -> (Wallet<CliWalletUtils>, PathBuf) {
    match try_load_pre_genesis_wallet(base_dir) {
        Ok(wallet) => wallet,
        Err(e) => {
            eprintln!("Error loading the wallet: {e}");
            safe_exit(1)
        }
    }
}
/// The default validator pre-genesis directory
pub fn validator_pre_genesis_dir(base_dir: &Path, alias: &str) -> PathBuf {
    base_dir.join(PRE_GENESIS_DIR).join(alias)
}

#[derive(Serialize)]
struct Bond {
    source: String,
    validator: String,
    amount: String,
}

#[derive(Serialize)]
struct BondList {
    bond: Vec<Bond>,
}

/// Sign genesis transactions.
pub async fn sign_genesis_tx(
    global_args: args::Global,
    args::SignGenesisTxs {
        source,
        validator,
        amount,
        validator_alias,
        use_device,
        device_transport,
    }: args::SignGenesisTxs,
) {
    let (wallet, _wallet_file) =
        load_pre_genesis_wallet_or_exit(&global_args.base_dir);
    let wallet_lock = RwLock::new(wallet);
    let maybe_pre_genesis_wallet = validator_alias.and_then(|alias| {
        let pre_genesis_dir =
            validator_pre_genesis_dir(&global_args.base_dir, &alias);
        pre_genesis::load(&pre_genesis_dir).ok()
    });
    let bond = Bond {
        source,
        validator,
        amount,
    };

    // Create the bond list
    let bond_list = BondList {
        bond: vec![bond],
    };

    // Serialize the bond list to a TOML string
    let toml_content = toml::to_string(&bond_list).unwrap_or_else(|err| {
        eprintln!("Unable to serialize to TOML. Failed with {err}.");
        safe_exit(1)
    });
    let contents = toml_content.into_bytes();
    // Sign a subset of the input txs (the ones whose keys we own)
    let unsigned = 
        genesis::transactions::parse_unsigned(&contents).unwrap();

    let signed = genesis::transactions::sign_txs(
        unsigned,
        &wallet_lock,
        maybe_pre_genesis_wallet.as_ref(),
        use_device,
        device_transport,
    )
    .await;

    let transactions = toml::to_string(&signed).unwrap();
    println!("{transactions}");
}

#[cfg(not(test))]
fn safe_exit(code: i32) -> ! {
    crate::cli::safe_exit(code)
}

#[cfg(test)]
fn safe_exit(code: i32) -> ! {
    panic!("Process exited unsuccessfully with error code: {}", code);
}
