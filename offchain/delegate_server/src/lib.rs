pub mod input_server;
pub mod output_server;
pub mod fee_manager_server;
pub mod rollups_server;

use offchain::fold::setup::{
    create_rollups_state_fold, create_input, create_output, create_fee_manager,
    RollupsStateFold, InputStateFold, OutputStateFold, FeeManagerStateFold,
};
use offchain::logic::instantiate_state_fold::{create_access, RollupsAccess};

use state_fold::config::SFConfig;

pub fn instantiate_input_fold_delegate(
    config: &SFConfig,
    url: String,
) -> InputStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_input(access, &config)
}

pub fn instantiate_output_fold_delegate(
    config: &SFConfig,
    url: String,
) -> OutputStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_output(access, &config)
}

pub fn instantiate_fee_manager_fold_delegate(
    config: &SFConfig,
    url: String,
) -> FeeManagerStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_fee_manager(access, &config)
}

pub fn instantiate_rollups_fold_delegate(
    config: &SFConfig,
    url: String,
) -> RollupsStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_rollups_state_fold(access, &config)
}
