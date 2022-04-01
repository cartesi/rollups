table! {
    notices (session_id, epoch_index, input_index, notice_index) {
        session_id -> Varchar,
        epoch_index -> Int4,
        input_index -> Int4,
        notice_index -> Int4,
        keccak -> Varchar,
        payload -> Nullable<Bytea>,
        timestamp -> Timestamptz,
    }
}
