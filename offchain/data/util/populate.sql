-- insert inputs
INSERT INTO inputs VALUES (0, 'msg-sender', 'tx-hash', 0, current_timestamp, 'input-0');
INSERT INTO inputs VALUES (1, 'msg-sender', 'tx-hash', 0, current_timestamp, 'input-1');
INSERT INTO inputs VALUES (2, 'msg-sender', 'tx-hash', 0, current_timestamp, 'input-2');

-- insert notices
INSERT INTO notices VALUES (0, 0, 'notice-0-0');
INSERT INTO notices VALUES (0, 1, 'notice-0-1');
INSERT INTO notices VALUES (0, 2, 'notice-0-2');
INSERT INTO notices VALUES (1, 0, 'notice-1-0');
INSERT INTO notices VALUES (1, 1, 'notice-1-1');
INSERT INTO notices VALUES (1, 2, 'notice-1-2');
INSERT INTO notices VALUES (2, 0, 'notice-2-0');
INSERT INTO notices VALUES (2, 1, 'notice-2-1');
INSERT INTO notices VALUES (2, 2, 'notice-2-2');

-- insert vouchers
INSERT INTO vouchers VALUES (0, 0, 'destination', 'voucher-0-0');
INSERT INTO vouchers VALUES (0, 1, 'destination', 'voucher-0-1');
INSERT INTO vouchers VALUES (0, 2, 'destination', 'voucher-0-2');
INSERT INTO vouchers VALUES (1, 0, 'destination', 'voucher-1-0');
INSERT INTO vouchers VALUES (1, 1, 'destination', 'voucher-1-1');
INSERT INTO vouchers VALUES (1, 2, 'destination', 'voucher-1-2');
INSERT INTO vouchers VALUES (2, 0, 'destination', 'voucher-2-0');
INSERT INTO vouchers VALUES (2, 1, 'destination', 'voucher-2-1');
INSERT INTO vouchers VALUES (2, 2, 'destination', 'voucher-2-2');

-- insert reports
INSERT INTO reports VALUES (0, 0, 'report-0-0');
INSERT INTO reports VALUES (0, 1, 'report-0-1');
INSERT INTO reports VALUES (0, 2, 'report-0-2');
INSERT INTO reports VALUES (1, 0, 'report-1-0');
INSERT INTO reports VALUES (1, 1, 'report-1-1');
INSERT INTO reports VALUES (1, 2, 'report-1-2');
INSERT INTO reports VALUES (2, 0, 'report-2-0');
INSERT INTO reports VALUES (2, 1, 'report-2-1');
INSERT INTO reports VALUES (2, 2, 'report-2-2');

INSERT INTO proofs VALUES (0, 0, 'voucher', 0, 0, '<hash>', '<hash>', '<hash>', '<hash>', ARRAY['<array>'::bytea], ARRAY['<array>'::bytea], '<context>');
INSERT INTO proofs VALUES (0, 0, 'notice', 0, 0, '<hash>', '<hash>', '<hash>', '<hash>', ARRAY['<array>'::bytea], ARRAY['<array>'::bytea], '<context>');
