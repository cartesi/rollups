pub mod input_server;
pub mod output_server;
pub mod rollups_server;

use offchain::fold::setup::{
    create_descartes_state_fold, create_input, create_output,
    DescartesStateFold, InputStateFold, OutputStateFold,
};
use offchain::logic::instantiate_state_fold::{create_access, DescartesAccess};

use state_fold::config::SFConfig;

pub fn instantiate_input_fold_delegate(
    config: &SFConfig,
    url: String,
) -> InputStateFold<DescartesAccess> {
    let access = create_access(config, url).unwrap();

    create_input(access, &config)
}

pub fn instantiate_output_fold_delegate(
    config: &SFConfig,
    url: String,
) -> OutputStateFold<DescartesAccess> {
    let access = create_access(config, url).unwrap();

    create_output(access, &config)
}

pub fn instantiate_descartes_fold_delegate(
    config: &SFConfig,
    url: String,
) -> DescartesStateFold<DescartesAccess> {
    let access = create_access(config, url).unwrap();

    create_descartes_state_fold(access, &config)
}
