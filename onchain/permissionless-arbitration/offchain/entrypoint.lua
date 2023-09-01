#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

local machine_path = "offchain/program/simple-program"
local ps_template = [[ps %s | grep defunct | wc -l]]

local log = require 'utils.log'
local Blockchain = require "blockchain.node"
local Machine = require "computation.machine"

local function is_zombie(pid)
    local reader = io.popen(string.format(ps_template, pid))
    ret = reader:read()
    reader:close()
    return tonumber(ret) == 1
end

local function stop_players(pid_reader)
    for pid, reader in pairs(pid_reader) do
        print(string.format("Stopping player with pid %s...", pid))
        os.execute(string.format("kill -15 %s", pid))
        reader:close()
        print "Player stopped"
    end

end

print "Hello, world!"
os.execute "cd offchain/program && ./gen_machine_simple.sh"

local m = Machine:new_from_path(machine_path)
local initial_hash = m:state().root_hash
local blockchain = Blockchain:new()
local contract = blockchain:deploy_contract(initial_hash)

-- add more player instances here
local cmds = {
    string.format([[sh -c "echo $$ ; exec ./offchain/player/honest_player.lua %d %s %s | tee honest.log"]], 1, contract, machine_path),
    string.format([[sh -c "echo $$ ; exec ./offchain/player/dishonest_player.lua %d %s %s %s | tee dishonest.log"]], 2, contract, machine_path, initial_hash)
}
local pid_reader = {}
local pid_player = {}

for i, cmd in ipairs(cmds) do
    local reader = io.popen(cmd)
    local pid = reader:read()
    pid_reader[pid] = reader
    pid_player[pid] = i
end

-- gracefully end children processes
setmetatable(pid_reader, {
    __gc = function(t)
        stop_players(t)
    end
})

local no_active_players = 0
while true do
    local last_ts = [[01/01/2000 00:00:00]]
    local players = 0

    for pid, reader in pairs(pid_reader) do
        players = players + 1
        if is_zombie(pid) then
            log.log(pid_player[pid], string.format("player process %s is dead", pid))
            reader:close()
            pid_reader[pid] = nil
        else
            last_ts = log.log_to_ts(reader, last_ts)
        end
    end

    if players == 0 then
        no_active_players = no_active_players + 1
    else
        no_active_players = 0
    end

    -- if no active player processes for 10 consecutive iterations, break loop
    if no_active_players == 10 then break end

    -- TODO: if all players are idle, advance anvil
end

print "Good-bye, world!"
