
CREATE TABLE "notices"
(
    id SERIAL,
    session_id character varying(255) NOT NULL,
    epoch_index int NOT NULL,
    input_index int NOT NULL,
    notice_index int NOT NULL,
    keccak character varying(255) NOT NULL,
    payload bytea,
    "timestamp" timestamp with time zone default current_timestamp NOT NULL,
    CONSTRAINT "notices_pkey" PRIMARY KEY (id, session_id, epoch_index, input_index, notice_index)
);


CREATE TABLE "state"
(
    "name" character varying(255) NOT NULL,
    "value_i32" int default 0 NOT NULL,
    CONSTRAINT "state_pkey" PRIMARY KEY ("name")
);

insert into "state" ("name", "value_i32") values ('current_epoch_index', 0);