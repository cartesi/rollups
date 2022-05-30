

CREATE TABLE "epochs"
(
    id SERIAL,
    epoch_index int NOT NULL,
    CONSTRAINT "epochs_pkey" PRIMARY KEY (id)
);


CREATE TABLE "inputs"
(
    id SERIAL,
    input_index int NOT NULL,
    epoch_index int NOT NULL,
    sender character varying(255) NOT NULL,
    tx_hash character varying(255) default NULL,
    block_number bigint NOT NULL,
    payload bytea NOT NULL,
    "timestamp" timestamp NOT NULL,
    CONSTRAINT "inputs_pkey" PRIMARY KEY (id)
);

CREATE TABLE "proofs"
(
    id SERIAL,
    output_hashes_root_hash character varying(255) NOT NULL,
    vouchers_epoch_root_hash character varying(255) NOT NULL,
    notices_epoch_root_hash character varying(255) NOT NULL,
    machine_state_hash character varying(255) NOT NULL,
    keccak_in_hashes_siblings text[] not NULL,
    output_hashes_in_epoch_siblings text[] not NULL,
    CONSTRAINT "proofs_pkey" PRIMARY KEY (id)
);

CREATE TABLE "notices"
(
    id SERIAL,
    session_id character varying(255) NOT NULL,
    epoch_index int NOT NULL,
    input_index int NOT NULL,
    notice_index int NOT NULL,
    proof_id int,
    keccak character varying(255) NOT NULL,
    payload bytea,
    CONSTRAINT "notices_pkey" PRIMARY KEY (id),
    CONSTRAINT "notices_proof_fkey" FOREIGN KEY (proof_id) REFERENCES proofs(id) ON DELETE SET NULL
);

CREATE TABLE "vouchers"
(
    id SERIAL,
    epoch_index int NOT NULL,
    input_index int NOT NULL,
    voucher_index int NOT NULL,
    proof_id int,
    destination character varying(255) NOT NULL,
    payload bytea,
    CONSTRAINT "vouchers_pkey" PRIMARY KEY (id),
    CONSTRAINT "vouchers_proof_fkey" FOREIGN KEY (proof_id) REFERENCES proofs(id) ON DELETE SET NULL
);

CREATE TABLE "reports"
(
    id SERIAL,
    epoch_index int NOT NULL,
    input_index int NOT NULL,
    report_index int NOT NULL,
    payload bytea,
    CONSTRAINT "reports_pkey" PRIMARY KEY (id)
);



CREATE TABLE "state"
(
    "name" character varying(255) NOT NULL,
    "value_i32" int default 0 NOT NULL,
    CONSTRAINT "state_pkey" PRIMARY KEY ("name")
);

insert into "state" ("name", "value_i32") values ('current_notice_epoch_index', 0);
insert into "state" ("name", "value_i32") values ('current_report_epoch_index', 0);
insert into "state" ("name", "value_i32") values ('current_input_epoch_index', 0);