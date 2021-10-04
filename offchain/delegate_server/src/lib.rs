pub mod input_server;
pub mod output_server;
pub mod rollups_server;

use ethers::providers::{Http, Provider};
use ethers::types::U64;
use std::convert::TryFrom;
use std::sync::Arc;

use offchain::fold::setup::{
    create_descartes_state_fold, create_input, create_output,
    DescartesStateFold, InputStateFold, OutputStateFold,
};
use state_fold::Access;

static HTTP_URL: &'static str = "http://localhost:8545";

pub fn instantiate_input_fold_delegate(
) -> InputStateFold<Access<Provider<Http>>> {
    let access = create_access();
    let sf_config = create_sf_config();

    create_input(access, &sf_config)
}

pub fn instantiate_output_fold_delegate(
) -> OutputStateFold<Access<Provider<Http>>> {
    let access = create_access();
    let sf_config = create_sf_config();

    create_output(access, &sf_config)
}

pub fn instantiate_descartes_fold_delegate(
) -> DescartesStateFold<Access<Provider<Http>>> {
    let access = create_access();
    let sf_config = create_sf_config();

    create_descartes_state_fold(access, &sf_config)
}

fn create_access() -> Arc<Access<Provider<Http>>> {
    let provider = Arc::new(Provider::<Http>::try_from(HTTP_URL).unwrap());

    Arc::new(Access::new(Arc::clone(&provider), U64::from(0), vec![], 4))
}

fn create_sf_config() -> state_fold::config::SFConfig {
    // WARNING: review these values before actually running
    state_fold::config::SFConfig {
        concurrent_events_fetch: 4,
        genesis_block: U64::from(0),
        query_limit_error_codes: vec![],
        safety_margin: 0,
    }
}
