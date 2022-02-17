use offchain::fold::types::{
    AccumulatingEpoch, EpochInputState, FinalizedEpoch, FinalizedEpochs,
    ImmutableState, Input, OutputState, PhaseState, RollupsState,
};
use offchain_core::types::Block;

use state_fold::types::BlockState;

use crate::error::*;

use ethers::types::{H256, U256, U64};

use diesel::deserialize::QueryableByName;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::row::NamedRow;
use diesel::sql_query;
use diesel::sql_types::{Array, Integer, Json, Timestamp, Uuid, VarChar};

use serde::{Deserialize, Serialize};

use core::str::FromStr;

use snafu::ResultExt;

use std::convert::TryFrom;
use std::time::SystemTime;

use im::Vector;

use crate::db::{val_to_hex_str, Connection, PollingPool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbInput {
    pub input: Input,
    pub id: uuid::Uuid,
}

impl DbInput {
    pub fn new(input: Input) -> Self {
        Self {
            input,
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn insert(
        &self,
        conn: &Connection,
        epoch_input_state_id: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let sender_string = val_to_hex_str(&self.input.sender);
        let timestamp_string = U256::to_string(&self.input.timestamp);

        let mut payloads = Vec::<String>::new();
        for payload in self.input.payload.as_ref() {
            payloads.push(val_to_hex_str(payload));
        }

        let timestamp = SystemTime::now();
        let query = sql_query(
            "INSERT INTO \"Inputs\" VALUES ($1, $2, $3, $4, $5, $6, $7);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(sender_string)
        .bind::<Array<VarChar>, _>(payloads)
        .bind::<VarChar, _>(timestamp_string)
        .bind::<Uuid, _>(epoch_input_state_id)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _ = query.execute(conn).context(DieselError)?;
        Ok(self.id)
    }
}

impl From<Input> for DbInput {
    fn from(input: Input) -> Self {
        DbInput::new(input)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbEpochInputState {
    pub epoch_input_state: EpochInputState,
    pub db_inputs: Vector<DbInput>,
    pub id: uuid::Uuid,
}

impl DbEpochInputState {
    pub fn new(
        epoch_input_state: EpochInputState,
        db_inputs: Vector<DbInput>,
    ) -> Self {
        Self {
            epoch_input_state,
            db_inputs,
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn insert(&self, conn: &Connection) -> Result<uuid::Uuid> {
        let epoch_number_i32 =
            i32::try_from(self.epoch_input_state.epoch_number.as_usize())
                .unwrap();

        let mut inputs = Vec::<uuid::Uuid>::new();
        for input in &self.db_inputs {
            inputs.push(input.insert(conn, self.id)?);
        }
        let timestamp = SystemTime::now();
        let query = sql_query(
            "INSERT INTO \"EpochInputStates\" VALUES ($1, $2, $3, $4);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<Integer, _>(epoch_number_i32)
        //.bind::<Array<Uuid>, _>(inputs)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _ = query.execute(conn).context(DieselError)?;
        Ok(self.id)
    }
}
impl From<EpochInputState> for DbEpochInputState {
    fn from(epoch_input_state: EpochInputState) -> Self {
        let mut db_inputs = Vector::<DbInput>::new();
        for input in &epoch_input_state.inputs {
            db_inputs.push_back(DbInput::from(input.clone()));
        }

        DbEpochInputState::new(epoch_input_state, db_inputs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbFinalizedEpoch {
    pub finalized_epoch: FinalizedEpoch,
    pub db_epoch_input_state: DbEpochInputState,
    id: uuid::Uuid,
}

impl DbFinalizedEpoch {
    pub fn new(
        finalized_epoch: FinalizedEpoch,
        db_epoch_input_state: DbEpochInputState,
    ) -> Self {
        Self {
            finalized_epoch,
            db_epoch_input_state,
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn insert(
        &self,
        conn: &Connection,
        finalized_epochs_id: uuid::Uuid,
    ) -> Result<()> {
        let epoch_number_str =
            U256::to_string(&self.finalized_epoch.epoch_number);
        let hash_str = H256::to_string(&self.finalized_epoch.hash);
        let finalized_block_hash_string =
            H256::to_string(&self.finalized_epoch.finalized_block_hash);
        let finalized_block_number_string =
            U64::to_string(&self.finalized_epoch.finalized_block_number);

        let inputs_uuid = self.db_epoch_input_state.insert(conn)?;

        let timestamp = SystemTime::now();

        let query = sql_query(
            "INSERT INTO \"FinalizedEpoches\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(epoch_number_str)
        .bind::<VarChar, _>(hash_str)
        .bind::<Uuid, _>(inputs_uuid)
        .bind::<VarChar, _>(finalized_block_hash_string)
        .bind::<VarChar, _>(finalized_block_number_string)
        .bind::<Uuid, _>(finalized_epochs_id)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _ = query.execute(conn);
        Ok(())
    }
}

impl From<FinalizedEpoch> for DbFinalizedEpoch {
    fn from(finalized_epoch: FinalizedEpoch) -> Self {
        let db_epoch_input_state =
            DbEpochInputState::from(finalized_epoch.inputs.clone());
        DbFinalizedEpoch::new(finalized_epoch, db_epoch_input_state)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbOutputState {
    output_state: OutputState,
    id: uuid::Uuid,
}

impl DbOutputState {
    pub fn new(output_state: OutputState) -> Self {
        Self {
            output_state,
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn insert(
        &self,
        conn: &Connection,
        rollups_hash: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let dapp_contract_address_str =
            val_to_hex_str(&self.output_state.dapp_contract_address);
        let vouchers_str =
            serde_json::to_string(&self.output_state.vouchers).unwrap();
        let vouchers_json = serde_json::Value::from_str(&vouchers_str).unwrap();
        let timestamp = SystemTime::now();
        let query = sql_query(
            "INSERT INTO \"OutputStates\" VALUES ($1, $2, $3, $4, $5, $6);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(dapp_contract_address_str)
        .bind::<Json, _>(vouchers_json)
        .bind::<Uuid, _>(rollups_hash)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _ = query.execute(conn).context(DieselError)?;
        Ok(self.id)
    }
}

impl From<OutputState> for DbOutputState {
    fn from(output_state: OutputState) -> Self {
        DbOutputState::new(output_state)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbFinalizedEpochs {
    pub finalized_epochs: FinalizedEpochs,
    pub db_finalized_epochs: Vector<DbFinalizedEpoch>,
    id: uuid::Uuid,
}

impl DbFinalizedEpochs {
    pub fn new(
        finalized_epochs: FinalizedEpochs,
        db_finalized_epochs: Vector<DbFinalizedEpoch>,
    ) -> Self {
        Self {
            finalized_epochs,
            db_finalized_epochs,
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn insert(
        &self,
        conn: &Connection,
        rollups_hash: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let intial_epoch_string =
            U256::to_string(&self.finalized_epochs.initial_epoch);
        let dapp_contract_address_string =
            val_to_hex_str(&self.finalized_epochs.dapp_contract_address);

        let timestamp = SystemTime::now();

        let query = sql_query(
            "INSERT INTO \"FinalizedEpochs\" VALUES ($1, $2, $3, $4, $5, $6);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(intial_epoch_string)
        .bind::<VarChar, _>(dapp_contract_address_string)
        .bind::<Uuid, _>(rollups_hash)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _ = query.execute(conn);

        for finalized_epoch in &self.db_finalized_epochs {
            finalized_epoch.insert(conn, self.id)?;
        }

        Ok(self.id)
    }
}

impl From<FinalizedEpochs> for DbFinalizedEpochs {
    fn from(finalized_epochs: FinalizedEpochs) -> Self {
        let mut db_finalized_epochs = Vector::<DbFinalizedEpoch>::new();
        for finalized_epoch in &finalized_epochs.finalized_epochs {
            db_finalized_epochs
                .push_back(DbFinalizedEpoch::from(finalized_epoch.clone()));
        }

        DbFinalizedEpochs::new(finalized_epochs, db_finalized_epochs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbAccumulatingEpoch {
    pub accumulating_epoch: AccumulatingEpoch,
    pub db_epoch_input_state: DbEpochInputState,
    id: uuid::Uuid,
}

impl DbAccumulatingEpoch {
    pub fn new(
        accumulating_epoch: AccumulatingEpoch,
        db_epoch_input_state: DbEpochInputState,
    ) -> Self {
        Self {
            accumulating_epoch,
            db_epoch_input_state,
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn insert(
        &self,
        conn: &Connection,
        rollups_uuid: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let epoch_number_string =
            U256::to_string(&self.accumulating_epoch.epoch_number);

        let dapp_contract_address_string =
            val_to_hex_str(&self.accumulating_epoch.dapp_contract_address);
        let epoch_input_state = self.db_epoch_input_state.insert(conn)?;

        let timestamp = SystemTime::now();

        let query = sql_query(
            "INSERT INTO \"AccumulatingEpoches\" VALUES ($1, $2, $3, $4, $5, \
                                                            $6, $7);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(epoch_number_string)
        .bind::<VarChar, _>(dapp_contract_address_string)
        .bind::<Uuid, _>(epoch_input_state)
        .bind::<Uuid, _>(rollups_uuid)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _rows_created = query.execute(conn).context(DieselError)?;
        Ok(self.id)
    }
}

impl From<AccumulatingEpoch> for DbAccumulatingEpoch {
    fn from(accumulating_epoch: AccumulatingEpoch) -> Self {
        let db_epoch_input_state =
            DbEpochInputState::from(accumulating_epoch.inputs.clone());

        DbAccumulatingEpoch::new(accumulating_epoch, db_epoch_input_state)
    }
}

use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbImmutableState {
    pub immutable_state: ImmutableState,
    id: uuid::Uuid,
}

impl DbImmutableState {
    fn new(immutable_state: ImmutableState) -> Self {
        Self {
            immutable_state,
            id: uuid::Uuid::new_v4(),
        }
    }

    fn insert(
        &self,
        conn: &Connection,
        rollups_uuid: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let input_duration_string =
            U256::to_string(&self.immutable_state.input_duration);
        let challenge_period_string =
            U256::to_string(&self.immutable_state.challenge_period);

        //Assuming that contract_creation_timestamp is in fact a 64 bit timestamp, we'll only have a
        //problem in the year 292.277.026.596
        let contract_creation_duration = Duration::from_secs(
            self.immutable_state.contract_creation_timestamp.as_u64(),
        );
        let contract_creation_timestamp =
            SystemTime::UNIX_EPOCH + contract_creation_duration;

        let dapp_contract_address_string =
            val_to_hex_str(&self.immutable_state.dapp_contract_address);

        let timestamp = SystemTime::now();
        let query = sql_query(
            "INSERT INTO \"ImmutableStates\" VALUES ($1, $2, $3, $4, $5, \
                                                            $6, $7, $8);;",
        )
        .bind::<Uuid, _>(self.id)
        .bind::<VarChar, _>(input_duration_string)
        .bind::<VarChar, _>(challenge_period_string)
        .bind::<Timestamp, _>(contract_creation_timestamp)
        .bind::<VarChar, _>(dapp_contract_address_string)
        .bind::<Uuid, _>(rollups_uuid)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp);

        let _rows_created = query.execute(conn).unwrap();
        Ok(self.id)
    }
}

impl From<ImmutableState> for DbImmutableState {
    fn from(immutable_state: ImmutableState) -> Self {
        DbImmutableState::new(immutable_state)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupsBlockHash {
    block_hash: H256,
}

impl QueryableByName<Pg> for RollupsBlockHash {
    fn build<R: NamedRow<Pg>>(row: &R) -> diesel::deserialize::Result<Self> {
        let block_hash_string: String = row.get("block_hash")?;
        Ok(Self {
            block_hash: H256::from_str(&block_hash_string)?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rollups {
    state: RollupsState,
    block_hash: H256,
    pub db_constants: DbImmutableState,
    pub db_finalized_epochs: DbFinalizedEpochs,
    pub db_current_epoch: DbAccumulatingEpoch,
    pub db_output_state: DbOutputState,
}

impl Rollups {
    pub fn new(
        state: RollupsState,
        block_hash: H256,
        db_constants: DbImmutableState,
        db_finalized_epochs: DbFinalizedEpochs,
        db_current_epoch: DbAccumulatingEpoch,
        db_output_state: DbOutputState,
    ) -> Self {
        Self {
            state,
            block_hash,
            db_constants,
            db_finalized_epochs,
            db_current_epoch,
            db_output_state,
        }
    }

    pub fn insert(
        &self,
        pool: &PollingPool,
        block: Block,
    ) -> std::result::Result<(), Error> {
        let id = uuid::Uuid::new_v4();
        let block_hash_string = val_to_hex_str(&block.hash);
        let query = sql_query(
            "SELECT * FROM \"DescartesV2States\" WHERE block_hash = $1;;",
        )
        .bind::<VarChar, _>(block_hash_string.clone());
        let conn = pool.get().context(R2D2Error)?;

        let states: Vec<RollupsBlockHash> = query.load(&conn).unwrap();
        if states.len() > 0 {
            return Ok(());
        }
        let phase_state = match &self.state.current_phase {
            PhaseState::InputAccumulation {} => "InputAccumulation",
            PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch: _ } => {
                "EpochSealedAwaitingFirstClaim"
            }
            PhaseState::AwaitingConsensusNoConflict { claimed_epoch: _ } => {
                "AwaitingConsensusNoConflict"
            }
            PhaseState::ConsensusTimeout { claimed_epoch: _ } => {
                "ConsensusTimeout"
            }
            PhaseState::AwaitingDispute { claimed_epoch: _ } => {
                "AwaitingDispute"
            }
            PhaseState::AwaitingConsensusAfterConflict {
                claimed_epoch: _,
                challenge_period_base_ts: _,
            } => "AwaitingConsensusAfterConflict",
        };

        let constants_id = self.db_constants.insert(&conn, id)?;
        let _ = self.db_finalized_epochs.insert(&conn, id)?;
        let current_epoch_id = self.db_current_epoch.insert(&conn, id)?;
        let output_state_id = self.db_output_state.insert(&conn, id)?;

        let initial_epoch_string = U256::to_string(&self.state.initial_epoch);

        let timestamp = SystemTime::now();
        let query = sql_query("INSERT INTO \"DescartesV2States\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind::<VarChar, _>(block_hash_string.clone())
            .bind::<Uuid, _>(constants_id)
            .bind::<VarChar, _>(initial_epoch_string)
            .bind::<Uuid, _>(current_epoch_id)
            .bind::<VarChar, _>(phase_state)
            .bind::<Uuid, _>(output_state_id)
            .bind::<Timestamp, _>(timestamp)
            .bind::<Timestamp, _>(timestamp)
            ;
        query.execute(&conn).context(DieselError)?;

        Ok(())
    }
}

impl From<BlockState<RollupsState>> for Rollups {
    fn from(block_state: BlockState<RollupsState>) -> Self {
        let rollups_state = block_state.state;
        let block_hash = block_state.block.hash;
        let db_constants =
            DbImmutableState::from(rollups_state.constants.clone());
        let db_finalized_epochs =
            DbFinalizedEpochs::from(rollups_state.finalized_epochs.clone());
        let db_current_epoch =
            DbAccumulatingEpoch::from(rollups_state.current_epoch.clone());
        let db_output_state =
            DbOutputState::from(rollups_state.output_state.clone());

        Rollups::new(
            rollups_state,
            block_hash,
            db_constants,
            db_finalized_epochs,
            db_current_epoch,
            db_output_state,
        )
    }
}
