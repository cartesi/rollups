local default_account_number = 10

local function stop_blockchain(handle, pid)
    print(string.format("Stopping blockchain with pid %d...", pid))
    os.execute(string.format("kill -15 %i", pid))
    handle:close()
    print "Blockchain stopped"
end

local function start_blockchain(account_num)
    account_num = account_num or default_account_number
    print(string.format("Starting blockchain with %d accounts...", account_num))

    local cmd = string.format([[sh -c "echo $$ ; exec anvil --block-time 1 -a %d"]], account_num)

    local reader = io.popen(cmd)
    assert(reader, "`popen` returned nil reader")

    local pid = tonumber(reader:read())

    local handle = {reader = reader, pid = pid}
    setmetatable(handle, {__gc = function(t)
        stop_blockchain(t.reader, t.pid)
    end})

    print(string.format("Blockchain running with pid %d", pid))
    return handle
end

local function capture_blockchain_data(reader, account_num)
    account_num = account_num or default_account_number
    local str

    local addresses = {}
    repeat
        str = reader:read();
        local _, _, address = str:find [[%(%d+%) ("0x%x+")]]
        if address then
            table.insert(addresses, address)
        end
    until str:find("Private Keys")
    assert(#addresses == account_num)

    local pks = {}
    repeat
        str = reader:read();
        local _, _, pk = str:find("%(%d+%) (0x%x+)")
        if pk then
            table.insert(pks, pk)
        end
    until str:find("Wallet")
    assert(#pks == account_num)

    local endpoint
    repeat
        str = reader:read();
        _, _, endpoint = str:find("Listening on ([%w%p]+)")
    until endpoint

    return {address = addresses, pk = pks}, endpoint
end


local function deploy_contracts(endpoint, deployer, initial_hash)

    -- Deploy Inner Factory
    print "Deploying inner factory..."

    local cmd_inner = string.format(
        [[sh -c "forge create InnerTournamentFactory --rpc-url=%s --private-key=%s"]],
        endpoint, deployer
    )

    local handle_inner = io.popen(cmd_inner)
    assert(handle_inner, "`popen` returned nil handle")

    local _, _, inner_factory_address = handle_inner:read("*a"):find("Deployed to: (0x%x+)")
    assert(inner_factory_address, "deployment failed, factory_address is nil")
    print("Inner factory deployed at", inner_factory_address)
    handle_inner:close()

    --
    -- Deploy Root Factory
    print "Deploying root factory..."

    local cmd_root = string.format(
        [[sh -c "forge create RootTournamentFactory --rpc-url=%s --private-key=%s --constructor-args %s"]],
        endpoint, deployer, inner_factory_address
    )

    local handle_root = io.popen(cmd_root)
    assert(handle_root, "`popen` returned nil handle")

    local _, _, root_factory_address = handle_root:read("*a"):find("Deployed to: (0x%x+)")
    assert(root_factory_address, "deployment failed, factory_address is nil")
    print("Root factory deployed at", root_factory_address)
    handle_root:close()


    --
    -- Instantiate Root Tournament
    print "Instantiate root contract..."

    local cmd2 = string.format(
        [[cast send --private-key "%s" --rpc-url "%s" "%s" "instantiateTopOfMultiple(bytes32)" "%s"]],
        deployer, endpoint, root_factory_address, initial_hash
    )

    local handle2 = io.popen(cmd2)
    assert(handle2, "`popen` returned nil handle")

    local _, _, a = handle2:read("*a"):find [["data":"0x000000000000000000000000(%x+)"]]
    local address = "0x" .. a
    assert(address, "deployment failed, address is nil")
    print("Contract deployed at", address)
    handle2:close()

    return address
end

local Blockchain = {}
Blockchain.__index = Blockchain

function Blockchain:new(account_num)
    local blockchain = {}

    local handle = start_blockchain(account_num)
    local accounts, endpoint = capture_blockchain_data(handle.reader, account_num)

    blockchain._handle = handle
    blockchain._accounts = accounts
    blockchain._current_account = 1
    blockchain.endpoint = "http://"..endpoint

    setmetatable(blockchain, self)
    return blockchain
end

function Blockchain:new_account()
    local current_account = self._current_account
    self._current_account = current_account + 1
    local accounts = self._accounts
    assert(current_account <= #accounts.address, "no more accounts")

    local account = {
        address = accounts.address[current_account],
        pk = accounts.pk[current_account]
    }

    return account
end

function Blockchain:deploy_contract(initial_hash, deployer)
    assert(initial_hash)
    deployer = deployer or self:new_account()
    local address = deploy_contracts(self.endpoint, deployer.pk, initial_hash)
    return address, deployer
end

function Blockchain:read_to(p)
    repeat until self._handle.reader:read():find(p)
end

-- local bc = Blockchain:new(100)
-- local initial_hash = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
-- bc:deploy_contract(initial_hash)

return Blockchain
