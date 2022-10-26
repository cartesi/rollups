# Processes forge test output and prints out a jq filter to be applied
# to the vouchers.json file in order to update the faulty proofs

/^[ ]*Proof for voucher [0-9]+ might be outdated/ {
    voucherFound = 1;
    voucherRow = NR;
    voucherId = $4;
}

voucherFound && (NR == voucherRow + 1) {
    destination[voucherId] = $1;
}

voucherFound && (NR == voucherRow + 2) {
    payload[voucherId] = $1;
}

END {
    printSeparator = 0;
    for (id in destination) {
        if (printSeparator) printf(" | ");
        printf(".[%s].destination = \"%s\" | .[%s].payload = \"%s\"",
               id, destination[id], id, payload[id]);
        printSeparator = 1;
    }
}
