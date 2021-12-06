use crate::db::PollingPool;
use crate::error::*;
use crate::grpc::state_server::GetStateResponse;
use crate::state::db::Rollups;

use state_fold::types::BlockState;

use offchain::fold::types::RollupsState;

use snafu::ResultExt;

#[derive(Clone)]
pub struct Writer {
    pub pool: PollingPool,
}

impl Writer {
    pub async fn consume(&self, response: GetStateResponse) -> Result<()> {
        let json_state = response.json_state;

        let block_state: BlockState<RollupsState> =
            serde_json::from_str(&json_state).context(DeserializeError)?;

        let rollups_block_state = Rollups::from(block_state.clone());
        rollups_block_state.insert(&self.pool, block_state.block)?;

        Ok(())
    }
}
