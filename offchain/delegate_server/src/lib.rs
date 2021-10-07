pub mod input_server;
pub mod output_server;
pub mod rollups_server;

use offchain::config::DescartesConfig;
use offchain::fold::setup::{
    create_descartes_state_fold, create_input, create_output,
    DescartesStateFold, InputStateFold, OutputStateFold,
};
use offchain::logic::instantiate_state_fold::{create_access, DescartesAccess};

pub fn instantiate_input_fold_delegate(
    config: &DescartesConfig,
) -> InputStateFold<DescartesAccess> {
    let access = create_access(config).unwrap();

    create_input(access, &config.sf_config)
}

pub fn instantiate_output_fold_delegate(
    config: &DescartesConfig,
) -> OutputStateFold<DescartesAccess> {
    let access = create_access(config).unwrap();

    create_output(access, &config.sf_config)
}

pub fn instantiate_descartes_fold_delegate(
    config: &DescartesConfig,
) -> DescartesStateFold<DescartesAccess> {
    let access = create_access(config).unwrap();

    create_descartes_state_fold(access, &config.sf_config)
}
