table! {
    epochs (id) {
        id -> Int4,
        epoch_index -> Int4,
    }
}

table! {
    inputs (id) {
        id -> Int4,
        input_index -> Int4,
        epoch_index -> Int4,
        sender -> Varchar,
        tx_hash -> Nullable<Varchar>,
        block_number -> Int8,
        payload -> Bytea,
        timestamp -> Timestamp,
    }
}

table! {
    notices (id) {
        id -> Int4,
        session_id -> Varchar,
        epoch_index -> Int4,
        input_index -> Int4,
        notice_index -> Int4,
        proof_id -> Nullable<Int4>,
        keccak -> Varchar,
        payload -> Nullable<Bytea>,
    }
}

table! {
    proofs (id) {
        id -> Int4,
        output_hashes_root_hash -> Varchar,
        vouchers_epoch_root_hash -> Varchar,
        notices_epoch_root_hash -> Varchar,
        machine_state_hash -> Varchar,
        keccak_in_hashes_siblings -> Array<Text>,
        output_hashes_in_epoch_siblings -> Array<Text>,
    }
}

table! {
    reports (id) {
        id -> Int4,
        epoch_index -> Int4,
        input_index -> Int4,
        report_index -> Int4,
        payload -> Nullable<Bytea>,
    }
}

table! {
    state (name) {
        name -> Varchar,
        value_i32 -> Int4,
    }
}

table! {
    vouchers (id) {
        id -> Int4,
        epoch_index -> Int4,
        input_index -> Int4,
        voucher_index -> Int4,
        proof_id -> Nullable<Int4>,
        destination -> Varchar,
        payload -> Nullable<Bytea>,
    }
}

joinable!(notices -> proofs (proof_id));
joinable!(vouchers -> proofs (proof_id));

allow_tables_to_appear_in_same_query!(
    epochs, inputs, notices, proofs, reports, state, vouchers,
);
