table! {
    epochs (epoch_index) {
        id -> Int4,
        epoch_index -> Int4,
    }
}

table! {
    inputs (id, input_index, epoch_index) {
        id -> Int4,
        input_index -> Int4,
        epoch_index -> Int4,
        sender -> Varchar,
        block_number -> Int8,
        payload -> Bytea,
        timestamp -> Timestamp,
    }
}

table! {
    notices (id, session_id, epoch_index, input_index, notice_index) {
        id -> Int4,
        session_id -> Varchar,
        epoch_index -> Int4,
        input_index -> Int4,
        notice_index -> Int4,
        keccak -> Varchar,
        payload -> Nullable<Bytea>,
        timestamp -> Timestamptz,
    }
}

table! {
    state (name) {
        name -> Varchar,
        value_i32 -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(epochs, inputs, notices, state,);
