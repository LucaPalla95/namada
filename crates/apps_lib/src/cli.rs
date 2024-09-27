//! The CLI commands that are re-used between the executables `namada`,
//! `namada-node` and `namada-client`.
//!
//! The `namada` executable groups together the most commonly used commands
//! inlined from the node and the client. The other commands for the node or the
//! client can be dispatched via `namada node ...` or `namada client ...`,
//! respectively.

pub mod api;
pub mod client;
pub mod context;
pub mod relayer;
mod utils;
pub mod wallet;

use clap::{ArgGroup, ArgMatches, ColorChoice};
use color_eyre::eyre::Result;
use utils::*;
pub use utils::{safe_exit, Cmd};

pub use self::context::Context;
use crate::cli::api::CliIo;

include!("../version.rs");

const APP_NAME: &str = "Namada";

// Main Namada sub-commands
const NODE_CMD: &str = "node";
const CLIENT_CMD: &str = "client";
const WALLET_CMD: &str = "wallet";
const RELAYER_CMD: &str = "relayer";

pub mod cmds {
    use super::utils::*;
    use super::{
        args, ArgMatches,
    };
    use crate::wrap;

    /// Commands for `namada` binary.
    #[allow(clippy::large_enum_variant)]
    #[derive(Clone, Debug)]
    pub enum Namada {
        // Sub-binary-commands
        Client(NamadaClient),
    }

    /// Used as top-level commands (`Cmd` instance) in `namadan` binary.
    /// Used as sub-commands (`SubCmd` instance) in `namada` binary.
    #[derive(Clone, Debug)]
    pub struct SignGenesisTxs(pub args::SignGenesisTxs);

    impl SubCmd for SignGenesisTxs {
        const CMD: &'static str = "sign-genesis-txs";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Self(args::SignGenesisTxs::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(wrap!("Sign genesis transaction(s)."))
                .add_args::<args::SignGenesisTxs>()
        }
    }
}

pub mod args {
    use std::env;
    use std::path::PathBuf;

    pub use namada_sdk::args::*;
    use namada_sdk::chain::ChainId;


    use namada_sdk::storage::BlockHeight;

    pub use namada_sdk::tx::{
        TX_BECOME_VALIDATOR_WASM, TX_BOND_WASM, TX_BRIDGE_POOL_WASM,
        TX_CHANGE_COMMISSION_WASM, TX_CHANGE_CONSENSUS_KEY_WASM,
        TX_CHANGE_METADATA_WASM, TX_CLAIM_REWARDS_WASM,
        TX_DEACTIVATE_VALIDATOR_WASM, TX_IBC_WASM, TX_INIT_ACCOUNT_WASM,
        TX_INIT_PROPOSAL, TX_REACTIVATE_VALIDATOR_WASM, TX_REDELEGATE_WASM,
        TX_RESIGN_STEWARD, TX_REVEAL_PK, TX_TRANSFER_WASM, TX_UNBOND_WASM,
        TX_UNJAIL_VALIDATOR_WASM, TX_UPDATE_ACCOUNT_WASM,
        TX_UPDATE_STEWARD_COMMISSION, TX_VOTE_PROPOSAL, TX_WITHDRAW_WASM,
        VP_USER_WASM,
    };

    use super::context::*;
    use super::utils::*;
    use super::ArgMatches;
    use crate::config;
    use crate::wrap;

    pub const AMOUNT_STR: Arg<String> = arg("amount");
    pub const BASE_DIR: ArgDefault<PathBuf> = arg_default(
        "base-dir",
        DefaultFn(|| match env::var("NAMADA_BASE_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => config::get_default_namada_folder(),
        }),
    );
    pub const SIGNATURES: ArgMulti<PathBuf, GlobStar> = arg_multi("signatures");
    pub const SOURCE_STR: Arg<String> = arg("source");
    pub const VALIDATOR_STR: Arg<String> = arg("validator");

    /// Global command arguments
    #[derive(Clone, Debug)]
    pub struct Global {
        pub is_pre_genesis: bool,
        pub chain_id: Option<ChainId>,
        pub base_dir: PathBuf,
        pub wasm_dir: Option<PathBuf>,
    }

    impl Global {
        /// Parse global arguments
        pub fn parse(matches: &ArgMatches) -> Self {
            let is_pre_genesis = PRE_GENESIS.parse(matches);
            let chain_id = CHAIN_ID_OPT.parse(matches);
            let base_dir = BASE_DIR.parse(matches);
            let wasm_dir = WASM_DIR.parse(matches);
            Global {
                is_pre_genesis,
                chain_id,
                base_dir,
                wasm_dir,
            }
        }

        /// Add global args definition. Should be added to every top-level
        /// command.
        pub fn def(app: App) -> App {
            app.next_line_help(true)
                .arg(
                    CHAIN_ID_OPT
                        .def()
                        .global(true)
                        .help(wrap!("The chain ID.")),
                )
                .arg(BASE_DIR.def().global(true).help(wrap!(
                    "The base directory is where the nodes, client and wallet \
                     configuration and state is stored. This value can also \
                     be set via `NAMADA_BASE_DIR` environment variable, but \
                     the argument takes precedence, if specified. Defaults to \
                     `$XDG_DATA_HOME/namada` (`$HOME/.local/share/namada` \
                     where `XDG_DATA_HOME` is unset) on \
                     Unix,`$HOME/Library/Application Support/Namada` on Mac, \
                     and `%AppData%\\Namada` on Windows."
                )))
                .arg(WASM_DIR.def().global(true).help(wrap!(
                    "Directory with built WASM validity predicates, \
                     transactions. This value can also be set via \
                     `NAMADA_WASM_DIR` environment variable, but the argument \
                     takes precedence, if specified."
                )))
                .arg(
                    PRE_GENESIS
                        .def()
                        .global(true)
                        .help(wrap!("Dispatch pre-genesis specific logic.")),
                )
        }
    }

    /// The concrete types being used in the CLI
    #[derive(Clone, Debug)]
    pub struct CliTypes;

    impl NamadaTypes for CliTypes {
        type AddrOrNativeToken = WalletAddrOrNativeToken;
        type Address = WalletAddress;
        type BalanceOwner = WalletBalanceOwner;
        type BlockHeight = BlockHeight;
        type BpConversionTable = PathBuf;
        type ConfigRpcTendermintAddress = ConfigRpcAddress;
        type Data = PathBuf;
        type DatedSpendingKey = WalletDatedSpendingKey;
        type DatedViewingKey = WalletDatedViewingKey;
        type EthereumAddress = String;
        type Keypair = WalletKeypair;
        type MaspIndexerAddress = String;
        type PaymentAddress = WalletPaymentAddr;
        type PublicKey = WalletPublicKey;
        type SpendingKey = WalletSpendingKey;
        type TendermintAddress = tendermint_rpc::Url;
        type TransferSource = WalletTransferSource;
        type TransferTarget = WalletTransferTarget;
        type ViewingKey = WalletViewingKey;
    }

    #[derive(Clone, Debug)]
    pub struct DefaultBaseDir {}

    impl Args for DefaultBaseDir {
        fn parse(_matches: &ArgMatches) -> Self {
            Self {}
        }

        fn def(app: App) -> App {
            app
        }
    }

    #[derive(Clone, Debug)]
    pub struct SignGenesisTxs {
        pub source: String,
        pub validator: String,
        pub amount: String,
        pub validator_alias: Option<String>,
        pub use_device: bool,
        pub device_transport: DeviceTransport,
    }

    impl Args for SignGenesisTxs {
        fn parse(matches: &ArgMatches) -> Self {
            let source = SOURCE_STR.parse(matches);
            let validator = VALIDATOR_STR.parse(matches);
            let amount = AMOUNT_STR.parse(matches);
            let validator_alias = ALIAS_OPT.parse(matches);
            let use_device = USE_DEVICE.parse(matches);
            let device_transport = DEVICE_TRANSPORT.parse(matches);
            Self {
                source,
                validator,
                amount,
                validator_alias,
                use_device,
                device_transport,
            }
        }

        fn def(app: App) -> App {
            app.arg(
                SOURCE_STR.def().help(wrap!(
                    "Path to the unsigned transactions TOML file."
                )),
            )
            .arg(VALIDATOR_STR.def().help(wrap!(
                "Save the output to a TOML file. When not supplied, the \
                 signed transactions will be printed to stdout instead."
            )))
            .arg(AMOUNT_STR.def().help(wrap!(
                "The amount of native token to transfer to the validator. \
                 This is a required parameter."
            )))
            .arg(
                ALIAS_OPT
                    .def()
                    .help(wrap!("Optional alias to a validator wallet.")),
            )
            .arg(USE_DEVICE.def().help(wrap!(
                "Derive an address and public key from the seed stored on the \
                 connected hardware wallet."
            )))
            .arg(DEVICE_TRANSPORT.def().help(wrap!(
                "Select transport for hardware wallet from \"hid\" (default) \
                 or \"tcp\"."
            )))
        }
    }
}

pub fn namada_cli() -> (cmds::Namada, String) {
    let app = namada_app();
    let matches = app.get_matches();
    let raw_sub_cmd =
        matches.subcommand().map(|(raw, _matches)| raw.to_string());
    let result = cmds::Namada::parse(&matches);
    match (result, raw_sub_cmd) {
        (Some(cmd), Some(raw_sub)) => return (cmd, raw_sub),
        _ => {
            namada_app().print_help().unwrap();
        }
    }
    safe_exit(2);
}

/// Namada client commands with loaded [`Context`] where required
pub enum NamadaClient {
    WithoutContext(Box<(cmds::ClientUtils, args::Global)>),
}

pub fn namada_client_cli() -> Result<NamadaClient> {
    let app = namada_client_app();
    let matches = app.clone().get_matches();
    match Cmd::parse(&matches) {
        Some(cmd) => {
            let global_args = args::Global::parse(&matches);
            match cmd {
                cmds::NamadaClient::WithContext(sub_cmd) => {
                    let context = Context::new::<CliIo>(global_args)?;
                    Ok(NamadaClient::WithContext(Box::new((sub_cmd, context))))
                }
                cmds::NamadaClient::WithoutContext(sub_cmd) => {
                    Ok(NamadaClient::WithoutContext(Box::new((
                        sub_cmd,
                        global_args,
                    ))))
                }
            }
        }
        None => {
            let mut app = app;
            app.print_help().unwrap();
            safe_exit(2);
        }
    }
}

pub fn namada_client_app() -> App {
    let app = App::new(APP_NAME)
        .version(namada_version())
        .about("Namada client command line interface.")
        .color(ColorChoice::Auto)
        .subcommand_required(true)
        .arg_required_else_help(true);
    cmds::NamadaClient::add_sub(args::Global::def(app))
}
