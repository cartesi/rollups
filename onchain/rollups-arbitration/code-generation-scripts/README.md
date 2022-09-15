# Code generation Scripts

Here we'll see some scripts in Lua language that actually generates some solidity smart contracts for the project. The files created are the enums for partition, epoch hash split, splice, memory manager and two party arbitration.

## Generation

If you wan to regenerate these files go to the root of the rollups-arbitration project and run:
```
lua scripts/generate_all_enums.lua --write
```