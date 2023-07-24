-- (c) Cartesi and individual authors (see AUTHORS)
-- SPDX-License-Identifier: Apache-2.0 (see LICENSE)

CREATE TABLE "inputs"
(
    "index" INT NOT NULL,
    "msg_sender" BYTEA NOT NULL,
    "tx_hash" BYTEA NOT NULL,
    "block_number" BIGINT NOT NULL,
    "timestamp" TIMESTAMP NOT NULL,
    "payload" BYTEA NOT NULL,
    CONSTRAINT "inputs_pkey" PRIMARY KEY ("index")
);

CREATE TABLE "vouchers"
(
    "input_index" INT NOT NULL,
    "index" INT NOT NULL,
    "destination" BYTEA NOT NULL,
    "payload" BYTEA NOT NULL,
    CONSTRAINT "vouchers_pkey" PRIMARY KEY ("input_index", "index"),
    CONSTRAINT "vouchers_input_index_fkey" FOREIGN KEY ("input_index") REFERENCES "inputs"("index")
);

CREATE TABLE "notices"
(
    "input_index" INT NOT NULL,
    "index" INT NOT NULL,
    "payload" BYTEA NOT NULL,
    CONSTRAINT "notices_pkey" PRIMARY KEY ("input_index", "index"),
    CONSTRAINT "notices_input_index_fkey" FOREIGN KEY ("input_index") REFERENCES "inputs"("index")
);

CREATE TABLE "reports"
(
    "input_index" INT NOT NULL,
    "index" INT NOT NULL,
    "payload" BYTEA NOT NULL,
    CONSTRAINT "reports_pkey" PRIMARY KEY ("input_index", "index"),
    CONSTRAINT "reports_input_index_fkey" FOREIGN KEY ("input_index") REFERENCES "inputs"("index")
);

CREATE TYPE "OutputEnum" AS ENUM ('voucher', 'notice');

CREATE TABLE "proofs"
(
    "input_index" INT NOT NULL,
    "output_index" INT NOT NULL,
    "output_enum" "OutputEnum" NOT NULL,
    "validity_input_index_within_epoch" INT NOT NULL,
    "validity_output_index_within_input" INT NOT NULL,
    "validity_output_hashes_root_hash" BYTEA NOT NULL,
    "validity_vouchers_epoch_root_hash" BYTEA NOT NULL,
    "validity_notices_epoch_root_hash" BYTEA NOT NULL,
    "validity_machine_state_hash" BYTEA NOT NULL,
    "validity_output_hash_in_output_hashes_siblings" BYTEA[] NOT NULL,
    "validity_output_hashes_in_epoch_siblings" BYTEA[] NOT NULL,
    "context" BYTEA NOT NULL,
    CONSTRAINT "proofs_pkey" PRIMARY KEY ("input_index", "output_index", "output_enum")
);
