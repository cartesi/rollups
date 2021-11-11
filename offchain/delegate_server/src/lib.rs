pub mod input_server;
pub mod voucher_server;
pub mod rollups_server;

use offchain::fold::setup::{
    create_rollups_state_fold, create_input, create_voucher,
    RollupsStateFold, InputStateFold, VoucherStateFold,
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

pub fn instantiate_voucher_fold_delegate(
    config: &SFConfig,
    url: String,
) -> VoucherStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_voucher(access, &config)
}

pub fn instantiate_rollups_fold_delegate(
    config: &SFConfig,
    url: String,
) -> RollupsStateFold<RollupsAccess> {
    let access = create_access(config, url).unwrap();

    create_rollups_state_fold(access, &config)
}
