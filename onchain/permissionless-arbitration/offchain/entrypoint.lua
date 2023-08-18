#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

print "Hello, world!"
os.execute "cd offchain/program && ./gen_machine_simple.sh"
local machine_path = "offchain/program/simple-program"

-- local Machine = require "computation.machine"
-- Machine:get_logs(machine_path, 0, 0)

-- os.exit()


local Player = require "player"
local Client = require "blockchain.client"

local Machine = require "computation.machine"
local m = Machine:new_from_path(machine_path)
local initial_hash = m:state().root_hash

local Blockchain = require "blockchain.node"
local blockchain = Blockchain:new()
local contract = blockchain:deploy_contract(initial_hash)


local p1
do
    local CommitmentBuilder = require "computation.commitment"
    local builder = CommitmentBuilder:new(machine_path)
    local client = Client:new(blockchain)
    p1 = Player:new(contract, client, builder, machine_path)
end

local p2
do
    local FakeCommitmentBuilder = require "computation.fake_commitment"

    -- m:run(m.start_cycle + 1)
    -- local second_hash = m:state().root_hash
    -- local builder = FakeCommitmentBuilder:new(initial_hash, second_hash)

    local builder = FakeCommitmentBuilder:new(initial_hash)
    local client = Client:new(blockchain)
    p2 = Player:new(contract, client, builder, machine_path)
end

local i = 0
while true do
    print(string.format("\n\n### ROUND %d ###\n", i))

    print "Player 1 react"
    if p1:react() then break end

    print ""

    print "Player 2 react"
    if p2:react() then break end

    i = i + 1
end











-- os.execute "jsonrpc-remote-cartesi-machine --server-address=localhost:8080 &"
-- os.execute "sleep 2"

-- require "cryptography.merkle_builder"
-- require "computation.commitment"
-- require "computation.machine_test"

-- local Blockchain = require "blockchain.node"

-- local bc = Blockchain:new(100)
-- local initial_hash = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
-- bc:deploy_contract(initial_hash)


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
