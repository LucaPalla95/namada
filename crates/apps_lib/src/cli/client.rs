use color_eyre::eyre::Result;
use namada_sdk::io::{display_line, Io, NamadaIo};
use namada_sdk::masp::ShieldedContext;
use namada_sdk::{Namada, NamadaImpl};

use crate::cli;
use crate::cli::api::{CliApi, CliClient};
use crate::cli::args::CliToSdk;
use crate::cli::cmds::*;
use crate::client::{rpc, tx, utils};

impl CliApi {
    pub async fn handle_client_command<C, IO: Io + Send + Sync>(
        client: Option<C>,
        cmd: cli::NamadaClient,
        io: IO,
    ) -> Result<()>
    where
        C: CliClient,
    {
        match cmd {
            cli::NamadaClient::WithoutContext(cmd_box) => {
                let (cmd, global_args) = *cmd_box;
                match cmd {
                    ClientUtils::SignGenesisTxs(SignGenesisTxs(args)) => {
                        utils::sign_genesis_tx(global_args, args).await
                    }
                }
            }
        }
        Ok(())
    }
}
