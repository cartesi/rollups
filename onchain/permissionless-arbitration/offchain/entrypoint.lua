#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

print "Hello, world!"
os.execute "cd program && ./gen_machine_simple.sh"

-- os.execute "jsonrpc-remote-cartesi-machine --server-address=localhost:8080 &"
-- os.execute "sleep 2"

-- require "cryptography.merkle_builder"
require "computation.commitment"
-- require "computation.machine_test"


-- local utils = require "utils"
-- local cartesi = {}
-- cartesi.rpc = require"cartesi.grpc"

-- local remote = cartesi.rpc.stub("localhost:8080", "localhost:8081")
-- local v = assert(remote.get_version())
-- print(string.format("Connected: remote version is %d.%d.%d\n", v.major, v.minor, v.patch))

-- local machine = remote.machine("program/simple-program")
-- print("cycles", machine:read_mcycle(), machine:read_uarch_cycle())
-- machine:snapshot()
-- machine:snapshot()

-- print(utils.hex_from_bin(machine:get_root_hash()))
-- machine:run(1000)
-- print(machine:read_iflags_H(), utils.hex_from_bin(machine:get_root_hash()))
-- machine:rollback()

-- print(utils.hex_from_bin(machine:get_root_hash()))
-- machine:run(1000)
-- print(machine:read_iflags_H(), utils.hex_from_bin(machine:get_root_hash()))
-- machine:rollback()

-- print(utils.hex_from_bin(machine:get_root_hash()))
-- machine:run(1000)
-- print(machine:read_iflags_H(), utils.hex_from_bin(machine:get_root_hash()))




-- machine:read_mcycle()



print "Good-bye, world!"
