use crate::db::{Connection, PollingPool};

use crate::error::*;

use crate::grpc::{
    cartesi_machine::{Hash, MerkleTreeProof},
    server_manager::{
        processed_input::ProcessedInputOneOf, Address, GetEpochStatusResponse,
        GetSessionStatusResponse, AcceptedData, Notice, ProcessedInput, Report,
        TaintStatus, Voucher,
    },
};

use crate::machine_manager::util::{
    convert_option, convert_row_string_to_u64, convert_vec,
    option_vec_u8_to_string, vec_u64_to_vec_string, vec_u8_to_string,
};

use serde::{Deserialize, Serialize};

use snafu::ResultExt;

use diesel::deserialize::QueryableByName;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::row::NamedRow;
use diesel::sql_query;
use diesel::sql_types::{Array, Json, Nullable, Timestamp, Uuid, VarChar};

use std::time::SystemTime;

use core::str::FromStr;

type DbReport = Vec<u8>;

impl From<Report> for DbReport {
    fn from(report: Report) -> Self {
        report.payload
    }
}

type DbHash = Vec<u8>;

impl From<Hash> for DbHash {
    fn from(hash: Hash) -> Self {
        hash.data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbMerkleTreeProof {
    id: uuid::Uuid,
    target_address: u64,
    log2_target_size: u64,
    target_hash: Option<DbHash>,
    log2_root_size: u64,
    root_hash: Option<DbHash>,
    sibling_hashes: Vec<DbHash>,
}

impl DbMerkleTreeProof {
    fn execute_insert_query(
        &self,
        conn: &Connection,
        id: uuid::Uuid,
        target_address: String,
        target_hash: String,
        log2_target_size: String,
        root_hash: String,
        log2_root_size: String,
        sibling_hashes: serde_json::Value,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query(
            "INSERT INTO \"MerkleTreeProofs\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);;",
        )
        .bind::<Uuid, _>(id)
        .bind::<VarChar, _>(target_address)
        .bind::<VarChar, _>(log2_target_size)
        .bind::<VarChar, _>(target_hash)
        .bind::<VarChar, _>(log2_root_size)
        .bind::<VarChar, _>(root_hash)
        .bind::<Json, _>(sibling_hashes)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp)
        .execute(conn)
        .context(DieselError)?;

        Ok(())
    }

    fn insert(&mut self, conn: &Connection) -> Result<Self> {
        let target_address_str = u64::to_string(&self.target_address);
        let log2_target_size_str = u64::to_string(&self.log2_target_size);
        let target_hash = option_vec_u8_to_string(self.target_hash.clone());
        let root_hash = option_vec_u8_to_string(self.root_hash.clone());
        let log2_root_size_str = u64::to_string(&self.log2_root_size);
        let sibling_hashes_json =
            serde_json::to_value(self.sibling_hashes.clone())
                .context(SerializeError)?;

        self.execute_insert_query(
            conn,
            self.id,
            target_address_str,
            target_hash,
            log2_target_size_str,
            root_hash,
            log2_root_size_str,
            sibling_hashes_json,
        )?;

        Ok(self.clone())
    }
}

impl QueryableByName<Pg> for DbMerkleTreeProof {
    fn build<R: NamedRow<Pg>>(row: &R) -> diesel::deserialize::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            target_address: convert_row_string_to_u64(row, "target_address")?,
            log2_target_size: convert_row_string_to_u64(
                row,
                "log2_target_size",
            )?,
            target_hash: row.get("target_hash")?,
            log2_root_size: convert_row_string_to_u64(row, "log2_root_size")?,
            root_hash: row.get("root_hash")?,
            sibling_hashes: row.get("sibling_hashes")?,
        })
    }
}

impl From<MerkleTreeProof> for DbMerkleTreeProof {
    fn from(proof: MerkleTreeProof) -> Self {
        let target_hash = convert_option(proof.target_hash);
        let root_hash = convert_option(proof.root_hash);
        let sibling_hashes = convert_vec(proof.sibling_hashes);

        Self {
            id: uuid::Uuid::new_v4(),
            target_address: proof.target_address,
            log2_target_size: proof.log2_target_size,
            target_hash,
            log2_root_size: proof.log2_root_size,
            root_hash,
            sibling_hashes,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbNotice {
    keccak: Option<DbHash>,
    payload: Vec<u8>,
    keccak_in_notice_hashes: Option<DbMerkleTreeProof>,
}

impl From<Notice> for DbNotice {
    fn from(notice: Notice) -> Self {
        let keccak = convert_option(notice.keccak);
        let keccak_in_notice_hashes =
            convert_option(notice.keccak_in_notice_hashes);

        Self {
            keccak,
            payload: notice.payload,
            keccak_in_notice_hashes,
        }
    }
}

impl DbNotice {
    fn execute_insert_query(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: String,
        input_index: String,
        notice_index: String,
        keccak: String,
        payload: String,
        keccak_in_notice_hashes: uuid::Uuid,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query(
            "INSERT INTO \"Notices\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);;",
        )
        .bind::<VarChar, _>(session_id.clone())
        .bind::<VarChar, _>(epoch_index)
        .bind::<VarChar, _>(input_index)
        .bind::<VarChar, _>(notice_index)
        .bind::<VarChar, _>(keccak)
        .bind::<VarChar, _>(payload)
        .bind::<Uuid, _>(keccak_in_notice_hashes)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp)
        .execute(conn)
        .context(DieselError)?;

        Ok(())
    }

    fn insert(
        &mut self,
        conn: &Connection,
        session_id: String,
        epoch_index: &u64,
        input_index: &u64,
        notice_index: &u64,
    ) -> Result<()> {
        let epoch_index_str = u64::to_string(epoch_index);
        let input_index_str = u64::to_string(input_index);
        let notice_index_str = u64::to_string(notice_index);
        let keccak_str = option_vec_u8_to_string(self.keccak.clone());
        let payload_str = vec_u8_to_string(self.payload.clone());
        let keccak_in_notice_hashes = match self.keccak_in_notice_hashes.clone()
        {
            Some(mut proof) => proof.insert(conn)?.id,
            None => uuid::Uuid::new_v4(),
        };

        self.execute_insert_query(
            conn,
            session_id,
            epoch_index_str,
            input_index_str,
            notice_index_str,
            keccak_str,
            payload_str,
            keccak_in_notice_hashes,
        )?;

        Ok(())
    }
}

pub type DbAddress = Vec<u8>;

impl From<Address> for DbAddress {
    fn from(address: Address) -> Self {
        address.data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbVoucher {
    keccak: Option<DbHash>,
    address: Option<DbAddress>,
    payload: Vec<u8>,
    keccak_in_voucher_hashes: Option<DbMerkleTreeProof>,
}

impl From<Voucher> for DbVoucher {
    fn from(voucher: Voucher) -> Self {
        let keccak = convert_option(voucher.keccak);
        let address = convert_option(voucher.address);
        let keccak_in_voucher_hashes =
            convert_option(voucher.keccak_in_voucher_hashes);

        Self {
            keccak,
            address,
            payload: voucher.payload,
            keccak_in_voucher_hashes,
        }
    }
}

impl DbVoucher {
    fn execute_insert_query(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: String,
        input_index: String,
        voucher_index: String,
        keccak: String,
        address: String,
        payload: String,
        keccak_in_voucher_hashes: uuid::Uuid,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query(
            "INSERT INTO \"Vouchers\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);;",
        )
        .bind::<VarChar, _>(session_id)
        .bind::<VarChar, _>(epoch_index)
        .bind::<VarChar, _>(input_index)
        .bind::<VarChar, _>(voucher_index)
        .bind::<VarChar, _>(keccak)
        .bind::<VarChar, _>(address)
        .bind::<VarChar, _>(payload)
        .bind::<Uuid, _>(keccak_in_voucher_hashes)
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp)
        .execute(conn)
        .context(DieselError)?;

        Ok(())
    }

    fn insert(
        &mut self,
        conn: &Connection,
        session_id: String,
        epoch_index: &u64,
        input_index: &u64,
        voucher_index: &u64,
    ) -> Result<()> {
        let epoch_index_str = u64::to_string(epoch_index);
        let input_index_str = u64::to_string(input_index);
        let voucher_index_str = u64::to_string(voucher_index);
        let keccak_str = option_vec_u8_to_string(self.keccak.clone());
        let address_str = option_vec_u8_to_string(self.address.clone());
        let payload_str = vec_u8_to_string(self.payload.clone());
        let keccak_in_voucher_hashes =
            match self.keccak_in_voucher_hashes.clone() {
                Some(mut proof) => proof.insert(conn)?.id,
                None => uuid::Uuid::new_v4(),
            };

        self.execute_insert_query(
            conn,
            session_id,
            epoch_index_str,
            input_index_str,
            voucher_index_str,
            keccak_str,
            address_str,
            payload_str,
            keccak_in_voucher_hashes,
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbInputResult {
    voucher_hashes_in_machine: Option<DbMerkleTreeProof>,
    vouchers: Vec<DbVoucher>,
    notice_hashes_in_machine: Option<DbMerkleTreeProof>,
    notices: Vec<DbNotice>,
}

impl From<AcceptedData> for DbInputResult {
    fn from(input_result: AcceptedData) -> Self {
        let voucher_hashes_in_machine =
            convert_option(input_result.voucher_hashes_in_machine);
        let vouchers = convert_vec(input_result.vouchers);
        let notice_hashes_in_machine =
            convert_option(input_result.notice_hashes_in_machine);
        let notices = convert_vec(input_result.notices);

        Self {
            voucher_hashes_in_machine,
            vouchers,
            notice_hashes_in_machine,
            notices,
        }
    }
}

impl DbInputResult {
    fn execute_insert_query(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: String,
        input_index: String,
        voucher_hashes_in_machine_id: uuid::Uuid,
        notice_hashes_in_machine_id: uuid::Uuid,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query(
            "INSERT INTO \"InputResults\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8);;",
        )
        .bind::<VarChar, _>(session_id.clone())
        .bind::<VarChar, _>(epoch_index)
        .bind::<VarChar, _>(input_index)
        .bind::<Uuid, _>(voucher_hashes_in_machine_id)
        .bind::<Uuid, _>(notice_hashes_in_machine_id)
        .bind::<Uuid, _>(uuid::Uuid::new_v4())
        .bind::<Timestamp, _>(timestamp)
        .bind::<Timestamp, _>(timestamp)
        .execute(conn)
        .context(DieselError)?;

        Ok(())
    }

    fn insert(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: &u64,
        input_index: &u64,
    ) -> Result<Self> {
        let epoch_index_str = u64::to_string(epoch_index);
        let input_index_str = u64::to_string(input_index);
        let voucher_hashes_in_machine_id =
            match self.voucher_hashes_in_machine.clone() {
                Some(mut proof) => proof.insert(conn)?.id,
                None => uuid::Uuid::new_v4(),
            };
        let notice_hashes_in_machine_id =
            match self.notice_hashes_in_machine.clone() {
                Some(mut proof) => proof.insert(conn)?.id,
                None => uuid::Uuid::new_v4(),
            };
        self.execute_insert_query(
            conn,
            session_id.clone(),
            epoch_index_str,
            input_index_str,
            voucher_hashes_in_machine_id,
            notice_hashes_in_machine_id,
        )?;

        for (i, voucher) in self.vouchers.iter().enumerate() {
            voucher.clone().insert(
                conn,
                session_id.clone(),
                epoch_index,
                input_index,
                &(i as u64),
            )?;
        }

        for (i, notice) in self.notices.iter().enumerate() {
            notice.clone().insert(
                conn,
                session_id.clone(),
                epoch_index,
                input_index,
                &(i as u64),
            )?;
        }

        Ok(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum DbProcessedOneof {
    Result(DbInputResult),
    SkipReason(Option<i32>),
}

impl DbProcessedOneof {
    fn insert(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: &u64,
        input_index: &u64,
    ) -> Result<Self> {
        match self.clone() {
            DbProcessedOneof::Result(input_result) => {
                Ok(DbProcessedOneof::Result(input_result.insert(
                    conn,
                    session_id,
                    epoch_index,
                    input_index,
                )?))
            }
            reason => Ok(reason),
        }
    }
}

impl From<ProcessedInputOneOf> for DbProcessedOneof {
    fn from(processed_oneof: ProcessedInputOneOf) -> Self {
        match processed_oneof {
            ProcessedInputOneOf::AcceptedData(db_input_result) => {
                DbProcessedOneof::Result(DbInputResult::from(db_input_result))
            }
            ProcessedInputOneOf::ExceptionData(num) => {
                DbProcessedOneof::SkipReason(Some(num))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbProcessedInput {
    input_index: u64,
    most_recent_machine_hash: Option<DbHash>,
    voucher_hashes_in_epoch: Option<DbMerkleTreeProof>,
    notice_hashes_in_epoch: Option<DbMerkleTreeProof>,
    reports: Vec<DbReport>,
    processed_oneof: DbProcessedOneof,
}

impl QueryableByName<Pg> for DbProcessedInput {
    fn build<R: NamedRow<Pg>>(row: &R) -> diesel::deserialize::Result<Self> {
        let input_index: String = row.get("input_index")?;

        Ok(Self {
            input_index: u64::from_str(&input_index).unwrap(),
            most_recent_machine_hash: None,
            voucher_hashes_in_epoch: None,
            notice_hashes_in_epoch: None,
            reports: Vec::<DbReport>::new(),
            processed_oneof: DbProcessedOneof::SkipReason(
                row.get("skip_reason")?,
            ),
        })
    }
}

impl DbProcessedInput {
    fn execute_insert_query(
        &self,
        conn: &Connection,
        session_id: String,
        epoch_index: String,
        input_index: String,
        most_recent_machine_hash: String,
        voucher_hashes_in_epoch_id: uuid::Uuid,
        notice_hashes_in_epoch_id: uuid::Uuid,
        reports_json: serde_json::Value,
        skip_reason: Option<String>,
    ) -> Result<()> {
        let timestamp = SystemTime::now();
        let insert_query = sql_query(
            "INSERT INTO \"ProcessedInputs\" VALUES ($1, $2, $3, $4,
                        $5, $6, $7, $8, $9, $10);;",
        )
        //this ID will be removed
        //.bind::<Uuid, _>(uuid::Uuid::new_v4())
        .bind::<VarChar, _>(session_id)
        .bind::<VarChar, _>(epoch_index)
        .bind::<VarChar, _>(input_index)
        .bind::<VarChar, _>(most_recent_machine_hash)
        .bind::<Uuid, _>(voucher_hashes_in_epoch_id)
        .bind::<Uuid, _>(notice_hashes_in_epoch_id)
        .bind::<Json, _>(reports_json);

        insert_query
            .bind::<Nullable<VarChar>, _>(skip_reason)
            .bind::<Timestamp, _>(timestamp)
            .bind::<Timestamp, _>(timestamp)
            .execute(conn)
            .context(DieselError)?;

        Ok(())
    }

    fn insert(
        &self,
        session_id: String,
        epoch_index: &u64,
        input_index: &u64,
        conn: &Connection,
    ) -> Result<()> {
        let epoch_index_str = u64::to_string(epoch_index);
        let input_index_str = u64::to_string(input_index);
        let most_recent_machine_hash =
            option_vec_u8_to_string(self.most_recent_machine_hash.clone());
        let voucher_hashes_in_epoch: Option<DbMerkleTreeProof> =
            convert_option(self.voucher_hashes_in_epoch.clone());
        let voucher_hashes_in_epoch_id = match voucher_hashes_in_epoch {
            Some(mut proof) => proof.insert(conn)?.id,
            None => uuid::Uuid::new_v4(),
        };

        let notice_hashes_in_epoch: Option<DbMerkleTreeProof> =
            convert_option(self.notice_hashes_in_epoch.clone());
        let notice_hashes_in_epoch_id = match notice_hashes_in_epoch {
            Some(mut proof) => proof.insert(conn)?.id,
            None => uuid::Uuid::new_v4(),
        };

        let reports: Vec<DbReport> = convert_vec(self.reports.clone());
        let reports_json =
            serde_json::to_value(reports).context(SerializeError)?;

        self.processed_oneof.clone().insert(
            conn,
            session_id.clone(),
            epoch_index,
            input_index,
        )?;

        let skip_reason: Option<String> = match self.processed_oneof {
            DbProcessedOneof::SkipReason(reason) => {
                Some(i32::to_string(&reason.unwrap()))
            }
            _ => None,
        };

        self.execute_insert_query(
            conn,
            session_id,
            epoch_index_str,
            input_index_str,
            most_recent_machine_hash,
            voucher_hashes_in_epoch_id,
            notice_hashes_in_epoch_id,
            reports_json,
            skip_reason,
        )?;

        Ok(())
    }
}

impl From<ProcessedInput> for DbProcessedInput {
    fn from(processed_input: ProcessedInput) -> Self {
        let most_recent_machine_hash =
            convert_option(processed_input.most_recent_machine_hash);
        let voucher_hashes_in_epoch =
            convert_option(processed_input.voucher_hashes_in_epoch);
        let notice_hashes_in_epoch =
            convert_option(processed_input.notice_hashes_in_epoch);
        let reports = convert_vec(processed_input.reports);
        let processed_oneof_opt =
            convert_option(processed_input.processed_oneof);
        let processed_oneof =
            processed_oneof_opt.expect("processed_oneof is None");

        Self {
            input_index: processed_input.input_index,
            most_recent_machine_hash,
            voucher_hashes_in_epoch,
            notice_hashes_in_epoch,
            reports,
            processed_oneof,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbEpochStatusResponse {
    session_id: String,
    epoch_index: u64,
    state: String,
    most_recent_machine_hash: Option<DbHash>,
    most_recent_vouchers_epoch_root_hash: Option<DbHash>,
    most_recent_notices_epoch_root_hash: Option<DbHash>,
    processed_inputs: Vec<DbProcessedInput>,
    pending_input_count: u64,
    taint_status: Option<DbTaintStatus>,
}

impl QueryableByName<Pg> for DbEpochStatusResponse {
    fn build<R: NamedRow<Pg>>(row: &R) -> diesel::deserialize::Result<Self> {
        let epoch_index_str: String = row.get("epoch_index")?;
        let pending_input_count_str: String = row.get("pending_input_count")?;
        let taint_status_json: Option<serde_json::Value> =
            row.get::<Nullable<Json>, _>("taint_status")?;

        let taint_status: Option<DbTaintStatus> = match taint_status_json {
            Some(json) => {
                if json == serde_json::Value::Null {
                    None
                } else {
                    Some(serde_json::from_value(json)?)
                }
            }
            None => None,
        };

        Ok(Self {
            session_id: row.get("session_id")?,
            epoch_index: u64::from_str(&epoch_index_str).unwrap(),
            state: row.get("state")?,
            most_recent_machine_hash: row.get("most_recent_machine_hash")?,
            most_recent_vouchers_epoch_root_hash: row
                .get("most_recent_vouchers_epoch_root_hash")?,
            most_recent_notices_epoch_root_hash: row
                .get("most_recent_notices_epoch_root_hash")?,
            processed_inputs: Vec::<DbProcessedInput>::new(),
            pending_input_count: u64::from_str(&pending_input_count_str)
                .unwrap(),
            taint_status: taint_status,
        })
    }
}

impl DbEpochStatusResponse {
    fn execute_select_processed_inputs_query(
        &self,
        conn: &Connection,
    ) -> Result<usize> {
        Ok(sql_query("SELECT * FROM \"ProcessedInputs\" WHERE session_id = $1 AND epoch_index = $2;;")
            .bind::<VarChar, _>(self.session_id.clone())
            .bind::<VarChar, _>(u64::to_string(&self.epoch_index))
            .execute(conn)
            .context(DieselError)?)
    }

    fn execute_select_epoch_query(
        &self,
        conn: &Connection,
    ) -> Result<Vec<Self>> {
        Ok(sql_query("SELECT * FROM \"EpochStatuses\" WHERE session_id = $1 AND epoch_index = $2;;")
            .bind::<VarChar, _>(self.session_id.clone())
            .bind::<VarChar, _>(u64::to_string(&self.epoch_index))
            .load(conn)
            .context(DieselError)?)
    }

    fn execute_update_query(
        &self,
        conn: &Connection,
        epoch_index: String,
        state: String,
        most_recent_machine_hash: String,
        most_recent_vouchers_epoch_root_hash: String,
        most_recent_notices_epoch_root_hash: String,
        pending_input_count: String,
        taint_status_json: serde_json::Value,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query("UPDATE \"EpochStatuses\" SET (state, most_recent_machine_hash, most_recent_vouchers_epoch_root_hash, most_recent_notices_epoch_root_hash, pending_input_count, taint_status, \"updatedAt\") = ($3, $4, $5, $6, $7, $8, $9) WHERE session_id=$1 AND epoch_index=$2;;")
            .bind::<VarChar, _>(self.session_id.clone())
            .bind::<VarChar, _>(epoch_index)
            .bind::<VarChar, _>(state)
            .bind::<VarChar, _>(most_recent_machine_hash)
            .bind::<VarChar, _>(most_recent_vouchers_epoch_root_hash)
            .bind::<VarChar, _>(most_recent_notices_epoch_root_hash)
            .bind::<VarChar, _>(pending_input_count)
            .bind::<Json, _>(taint_status_json)
            .bind::<Timestamp, _>(timestamp)
            .execute(conn)
            .context(DieselError)?;

        Ok(())
    }

    fn execute_insert_query(
        &self,
        conn: &Connection,
        epoch_index: String,
        state: String,
        most_recent_machine_hash: String,
        most_recent_vouchers_epoch_root_hash: String,
        most_recent_notices_epoch_root_hash: String,
        pending_input_count: String,
        taint_status_json: serde_json::Value,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        sql_query("INSERT INTO \"EpochStatuses\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);;")
            .bind::<VarChar, _>(self.session_id.clone())
            .bind::<VarChar, _>(epoch_index)
            .bind::<VarChar, _>(state)
            .bind::<VarChar, _>(most_recent_machine_hash)
            .bind::<VarChar, _>(most_recent_vouchers_epoch_root_hash)
            .bind::<VarChar, _>(most_recent_notices_epoch_root_hash)
            .bind::<VarChar, _>(pending_input_count)
            .bind::<Json, _>(taint_status_json)
            .bind::<Timestamp, _>(timestamp)
            .bind::<Timestamp, _>(timestamp)
            .execute(conn)
            .context(DieselError)?;

        Ok(())
    }

    pub fn insert(&mut self, pool: &PollingPool) -> Result<Self> {
        let conn = pool.get().context(R2D2Error)?;

        let epoch_index_str = u64::to_string(&self.epoch_index);
        let most_recent_machine_hash =
            option_vec_u8_to_string(self.most_recent_machine_hash.clone());
        let most_recent_vouchers_epoch_root_hash = option_vec_u8_to_string(
            self.most_recent_vouchers_epoch_root_hash.clone(),
        );
        let most_recent_notices_epoch_root_hash = option_vec_u8_to_string(
            self.most_recent_notices_epoch_root_hash.clone(),
        );
        let pending_input_count_str = u64::to_string(&self.pending_input_count);
        let taint_status_json =
            serde_json::to_value(&self.taint_status).context(SerializeError)?;

        let epoch_statuses = self.execute_select_epoch_query(&conn)?;

        if let Some(_epoch_status) = epoch_statuses.first() {
            let num_processed_inputs_db =
                self.execute_select_processed_inputs_query(&conn)?;

            if num_processed_inputs_db > self.processed_inputs.len() {
                return Err(Error::OutOfSync {
                    err: format!(
                        "Server Manager processed {}, should've processed {}",
                        self.processed_inputs.len(),
                        num_processed_inputs_db
                    ),
                });
            }
            let to_be_processed_inputs =
                &self.processed_inputs[num_processed_inputs_db..];
            for next_db_input in to_be_processed_inputs.iter() {
                next_db_input.insert(
                    self.session_id.clone(),
                    &self.epoch_index,
                    &next_db_input.input_index,
                    &conn,
                )?;
            }

            self.execute_update_query(
                &conn,
                epoch_index_str,
                self.state.clone(),
                most_recent_machine_hash,
                most_recent_vouchers_epoch_root_hash,
                most_recent_notices_epoch_root_hash,
                pending_input_count_str,
                taint_status_json,
            )?;
        } else {
            self.execute_insert_query(
                &conn,
                epoch_index_str,
                self.state.clone(),
                most_recent_machine_hash,
                most_recent_vouchers_epoch_root_hash,
                most_recent_notices_epoch_root_hash,
                pending_input_count_str,
                taint_status_json,
            )?;

            for input in self.processed_inputs.iter() {
                input.insert(
                    self.session_id.clone(),
                    &self.epoch_index,
                    &input.input_index,
                    &conn,
                )?;
            }
        }

        Ok(self.clone())
    }
}

impl From<GetEpochStatusResponse> for DbEpochStatusResponse {
    fn from(epoch_status_response: GetEpochStatusResponse) -> Self {
        let most_recent_machine_hash =
            convert_option(epoch_status_response.most_recent_machine_hash);
        let most_recent_vouchers_epoch_root_hash = convert_option(
            epoch_status_response.most_recent_vouchers_epoch_root_hash,
        );
        let most_recent_notices_epoch_root_hash = convert_option(
            epoch_status_response.most_recent_notices_epoch_root_hash,
        );
        let processed_inputs =
            convert_vec(epoch_status_response.processed_inputs);
        let taint_status = convert_option(epoch_status_response.taint_status);

        let mut state = String::from("Finished");
        if epoch_status_response.state != 1 {
            state = String::from("Active");
        }

        Self {
            session_id: epoch_status_response.session_id,
            epoch_index: epoch_status_response.epoch_index,
            state: state,
            most_recent_machine_hash,
            most_recent_vouchers_epoch_root_hash,
            most_recent_notices_epoch_root_hash,
            processed_inputs,
            pending_input_count: epoch_status_response.pending_input_count,
            taint_status,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbTaintStatus {
    pub error_code: i32,
    pub error_message: String,
}

impl From<TaintStatus> for DbTaintStatus {
    fn from(taint_status: TaintStatus) -> Self {
        Self {
            error_code: taint_status.error_code,
            error_message: taint_status.error_message,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbSessionStatusResponse {
    pub session_id: String,
    pub active_epoch_index: u64,
    pub epoch_index: Vec<u64>,
    pub taint_status: Option<DbTaintStatus>,
}

impl DbSessionStatusResponse {
    fn execute_insert_query(
        self,
        conn: &Connection,
        session_id: String,
        active_epoch_index: String,
        epoch_index: Vec<String>,
        taint_status_json: serde_json::Value,
    ) -> Result<()> {
        let timestamp = SystemTime::now();

        let insert_query = sql_query(
            "INSERT INTO \"SessionStatuses\" VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (\"session_id\")
            DO
                UPDATE SET (active_epoch_index, epoch_index, taint_status, \"updatedAt\") = ($2, $3, $4, $5);;
            "
            );

        insert_query
            .bind::<VarChar, _>(session_id)
            .bind::<VarChar, _>(active_epoch_index)
            .bind::<Array<VarChar>, _>(epoch_index)
            .bind::<Json, _>(taint_status_json)
            .bind::<Timestamp, _>(timestamp)
            .bind::<Timestamp, _>(timestamp)
            .execute(conn)
            .context(DieselError)?;
        Ok(())
    }

    pub fn insert(self, pool: &PollingPool) -> Result<()> {
        let conn = pool.get().context(R2D2Error)?;

        let session_id = self.session_id.clone();
        let active_epoch_index_str = u64::to_string(&self.active_epoch_index);
        let taint_status_json = serde_json::to_value(self.taint_status.clone())
            .context(SerializeError)?;

        let epoch_index_str = vec_u64_to_vec_string(self.epoch_index.clone());

        self.execute_insert_query(
            &conn,
            session_id,
            active_epoch_index_str,
            epoch_index_str,
            taint_status_json,
        )?;

        Ok(())
    }
}

impl From<GetSessionStatusResponse> for DbSessionStatusResponse {
    fn from(session_status: GetSessionStatusResponse) -> Self {
        let taint_status = match session_status.taint_status {
            Some(status) => Some(DbTaintStatus::from(status)),
            None => None,
        };
        Self {
            session_id: session_status.session_id,
            active_epoch_index: session_status.active_epoch_index,
            epoch_index: session_status.epoch_index,
            taint_status,
        }
    }
}
