#![allow(non_snake_case)]
// Generated with diesel cli
// diesel --database-url "postgres://<username>:<password>@localhost/postgres" print-schema > db_schema.rs

use diesel::table;

table! {
    AccumulatingEpoches (id) {
        id -> Uuid,
        epoch_number -> Varchar,
        descartesv2_contract_address -> Varchar,
        input_contract_address -> Varchar,
        epochInputStateId -> Uuid,
        rollups_hash -> Nullable<Varchar>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    DescartesV2States (block_hash) {
        block_hash -> Varchar,
        constants -> Uuid,
        initial_epoch -> Varchar,
        current_epoch -> Uuid,
        current_phase -> Varchar,
        voucher_state -> Uuid,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    EpochInputStates (id) {
        id -> Uuid,
        epoch_number -> Int4,
        input_contract_address -> Varchar,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    EpochStatuses (session_id, epoch_index) {
        session_id -> Varchar,
        epoch_index -> Varchar,
        state -> Varchar,
        most_recent_machine_hash -> Varchar,
        most_recent_vouchers_epoch_root_hash -> Varchar,
        most_recent_notices_epoch_root_hash -> Varchar,
        pending_input_count -> Varchar,
        taint_status -> Json,
        createdAt -> Nullable<Timestamptz>,
        updatedAt -> Nullable<Timestamptz>,
    }
}

table! {
    FinalizedEpoches (id) {
        id -> Uuid,
        epoch_number -> Varchar,
        hash -> Varchar,
        epochInputStateId -> Uuid,
        finalized_block_hash -> Varchar,
        finalized_block_number -> Varchar,
        FinalizedEpochId -> Uuid,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    FinalizedEpochs (id) {
        id -> Uuid,
        initial_epoch -> Varchar,
        descartesv2_contract_address -> Varchar,
        input_contract_address -> Varchar,
        rollups_hash -> Nullable<Varchar>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    ImmutableStates (id) {
        id -> Uuid,
        input_duration -> Varchar,
        challenge_period -> Varchar,
        contract_creation_timestamp -> Timestamptz,
        input_contract_address -> Varchar,
        output_contract_address -> Varchar,
        validator_contract_address -> Varchar,
        dispute_contract_address -> Varchar,
        descartesv2_contract_address -> Varchar,
        rollups_hash -> Nullable<Varchar>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    InputResults (session_id, epoch_index, input_index) {
        session_id -> Varchar,
        epoch_index -> Varchar,
        input_index -> Varchar,
        voucher_hashes_in_machine -> Uuid,
        notice_hashes_in_machine -> Uuid,
        processed_input_id -> Nullable<Uuid>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    Inputs (id) {
        id -> Uuid,
        sender -> Varchar,
        payload -> Nullable<Array<Text>>,
        timestamp -> Varchar,
        epoch_input_state_id -> Uuid,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    MerkleTreeProofs (id) {
        id -> Uuid,
        target_address -> Varchar,
        log2_target_size -> Varchar,
        target_hash -> Varchar,
        log2_root_size -> Varchar,
        root_hash -> Varchar,
        sibling_hashes -> Json,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    Notices (session_id, epoch_index, input_index, notice_index) {
        session_id -> Varchar,
        epoch_index -> Varchar,
        input_index -> Varchar,
        notice_index -> Varchar,
        keccak -> Nullable<Varchar>,
        payload -> Nullable<Text>,
        keccak_in_notice_hashes -> Uuid,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    OutputStates (id) {
        id -> Uuid,
        output_address -> Varchar,
        vouchers -> Json,
        rollups_hash -> Nullable<Varchar>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    ProcessedInputs (session_id, epoch_index, input_index) {
        session_id -> Varchar,
        epoch_index -> Varchar,
        input_index -> Varchar,
        most_recent_machine_hash -> Varchar,
        voucher_hashes_in_epoch -> Uuid,
        notice_hashes_in_epoch -> Uuid,
        reports -> Nullable<Json>,
        skip_reason -> Nullable<Varchar>,
        createdAt -> Nullable<Timestamptz>,
        updatedAt -> Nullable<Timestamptz>,
    }
}

table! {
    Reports (id) {
        id -> Uuid,
        payload -> Nullable<Text>,
        processed_input_id -> Nullable<Uuid>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    SequelizeMeta (name) {
        name -> Varchar,
    }
}

table! {
    SessionStatuses (session_id) {
        session_id -> Varchar,
        active_epoch_index -> Varchar,
        epoch_index -> Array<Varchar>,
        taint_status -> Json,
        createdAt -> Nullable<Timestamptz>,
        updatedAt -> Nullable<Timestamptz>,
    }
}

table! {
    Versions (id) {
        id -> Int4,
        version -> Nullable<Varchar>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

table! {
    Vouchers (session_id, epoch_index, input_index, voucher_index) {
        session_id -> Varchar,
        epoch_index -> Varchar,
        input_index -> Varchar,
        voucher_index -> Varchar,
        keccak -> Nullable<Varchar>,
        Address -> Varchar,
        payload -> Nullable<Text>,
        keccak_in_voucher_hashes -> Uuid,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    AccumulatingEpoches,
    DescartesV2States,
    EpochInputStates,
    EpochStatuses,
    FinalizedEpoches,
    FinalizedEpochs,
    ImmutableStates,
    InputResults,
    Inputs,
    MerkleTreeProofs,
    Notices,
    OutputStates,
    ProcessedInputs,
    Reports,
    SequelizeMeta,
    SessionStatuses,
    Versions,
    Vouchers,
);
