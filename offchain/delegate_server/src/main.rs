#![warn(unused_extern_crates)]
use state_server_grpc::{serve_delegate_manager, wait_for_signal};

use structopt::StructOpt;
use tokio::sync::oneshot;

use configuration::config::{
    Config as BasicConfig, EnvCLIConfig as BasicEnvCLIConfig,
};
use state_fold::config::{SFConfig, SFEnvCLIConfig};

const SERVER_ADDRESS: &str = "0.0.0.0:50051";

#[derive(StructOpt)]
struct ServerConfig {
    #[structopt(flatten)]
    sf_cli_config: SFEnvCLIConfig,
    #[structopt(flatten)]
    basic_cli_config: BasicEnvCLIConfig,
    #[structopt(flatten)]
    server_type: DelegateServerType,
}

#[derive(StructOpt)]
enum DelegateServerType {
    Input,
    Voucher,
    FeeManager,
    Rollups,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let _ = tokio::spawn(wait_for_signal(shutdown_tx));

    let server_config = ServerConfig::from_args();
    let sf_config = SFConfig::initialize(server_config.sf_cli_config).unwrap();
    let basic_config =
        BasicConfig::initialize(server_config.basic_cli_config).unwrap();

    match server_config.server_type {
        DelegateServerType::Input => {
            let input_fold = delegate_server::instantiate_input_fold_delegate(
                &sf_config,
                basic_config.url.clone(),
            );

            serve_delegate_manager(
                SERVER_ADDRESS,
                delegate_server::input_server::InputDelegateManager {
                    fold: input_fold,
                },
                shutdown_rx,
            )
            .await
        }
        DelegateServerType::Output => {
            let output_fold = delegate_server::instantiate_output_fold_delegate(
                &sf_config,
                basic_config.url.clone(),
            );

            serve_delegate_manager(
                SERVER_ADDRESS,
                delegate_server::output_server::OutputDelegateManager {
                    fold: output_fold,
                },
                shutdown_rx,
            )
            .await
        }
        DelegateServerType::FeeManager => {
            let fee_manager_fold = delegate_server::instantiate_fee_manager_fold_delegate(
                &sf_config,
                basic_config.url.clone(),
            );

            serve_delegate_manager(
                "[::1]:50051",
                delegate_server::fee_manager_server::FeeManagerDelegateManager {
                    fold: fee_manager_fold,
                },
                shutdown_rx,
            )
            .await
        }
        DelegateServerType::Rollups => {
            let rollups_fold =
                delegate_server::instantiate_rollups_fold_delegate(
                    &sf_config,
                    basic_config.url.clone(),
                );

            serve_delegate_manager(
                SERVER_ADDRESS,
                delegate_server::rollups_server::RollupsDelegateManager {
                    fold: rollups_fold,
                },
                shutdown_rx,
            )
            .await
        }
    }
}
