#!/bin/bash
cartesi-machine \
  --no-root-flash-drive \
  --rom-image="./bins/bootstrap.bin" \
  --ram-image="./bins/rv64ui-p-addi.bin" \
  --uarch-ram-image="/usr/share/cartesi-machine/uarch/uarch-ram.bin" \
  --uarch-ram-length=0x1000000 \
  --max-mcycle=0 \
  --store="simple-program"
