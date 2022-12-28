# Processes forge test output and prints out a jq filter to be applied
# to the vouchers.json file in order to update the faulty proofs

/^[ ]*Proof for output [0-9]+ might be outdated/ {
    voucherFound = 1;
    voucherRow = NR;
    voucherId = $4;
}

voucherFound && (NR == voucherRow + 1) {
    sender[voucherId] = $1;
}

voucherFound && (NR == voucherRow + 2) {
    payload[voucherId] = $1;
}

END {
    printSeparator = 0;
    for (id in sender) {
        if (printSeparator) printf(" | ");
        printf(".[%s].sender = \"%s\" | .[%s].payload = \"%s\"",
               id, sender[id], id, payload[id]);
        printSeparator = 1;
    }
}
