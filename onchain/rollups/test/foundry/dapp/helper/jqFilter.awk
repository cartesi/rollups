# Processes forge test output and prints out a jq filter to be applied
# to the inputs.json file in order to update the faulty proofs

/^[ ]*Proof for output [0-9]+ might be outdated/ {
    outputFound = 1;
    outputRow = NR;
    outputId = $4;
}

outputFound && (NR == outputRow + 1) {
    sender[outputId] = $1;
}

outputFound && (NR == outputRow + 2) {
    payload[outputId] = $1;
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
