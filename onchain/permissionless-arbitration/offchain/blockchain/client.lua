local Hash = require "cryptography.hash"
local MerkleTree = require "cryptography.merkle_tree"
local eth_ebi = require "utils.eth_ebi"

local function parse_topics(json)
    local _, _, topics = json:find(
        [==["topics":%[([^%]]*)%]]==]
    )

    local t = {}
    for k, _ in string.gmatch(topics, [["(0x%x+)"]]) do
        table.insert(t, k)
    end

    return t
end

local function parse_data(json, sig)
    local _, _, data = json:find(
        [==["data":"(0x%x+)"]==]
    )

    local decoded_data = eth_ebi.decode_event_data(sig, data)
    return decoded_data
end

local function parse_meta(json)
    local _, _, block_hash = json:find(
        [==["blockHash":"(0x%x+)"]==]
    )

    local _, _, block_number = json:find(
        [==["blockNumber":"(0x%x+)"]==]
    )

    local _, _, log_index = json:find(
        [==["logIndex":"(0x%x+)"]==]
    )

    local t = {
        block_hash = block_hash,
        block_number = tonumber(block_number),
        log_index = tonumber(log_index),
    }

    return t
end


local function parse_logs(logs, data_sig)
    local ret = {}
    for k, _ in string.gmatch(logs, [[{[^}]*}]]) do
        local emited_topics = parse_topics(k)
        local decoded_data = parse_data(k, data_sig)
        local meta = parse_meta(k)
        table.insert(ret, { emited_topics = emited_topics, decoded_data = decoded_data, meta = meta })
    end

    return ret
end

local function join_tables(...)
    local function join(ret, t, ...)
        if not t then return ret end

        for k, v in ipairs(t) do
            ret[k] = v
        end

        return join(ret, ...)
    end

    local ret = join({}, ...)
    return ret
end

local function sort_and_dedup(t)
    table.sort(t, function(a, b)
        local m1, m2 = a.meta, b.meta

        if m1.block_number < m2.block_number then
            return true
        elseif m1.block_number > m2.block_number then
            return false
        else
            if m1.log_index <= m2.log_index then
                return true
            else
                return false
            end
        end
    end)

    local ret = {}
    for k, v in ipairs(t) do
        local v2 = t[k + 1]
        if not v2 then
            table.insert(ret, v)
        else
            local m1, m2 = v.meta, v2.meta
            if not (m1.block_number == m2.block_number and m1.log_index == m2.log_index) then
                table.insert(ret, v)
            end
        end
    end

    return ret
end

local function quote_args(args, not_quote)
    local quoted_args = {}
    for _, v in ipairs(args) do
        if type(v) == "table" and (getmetatable(v) == Hash or getmetatable(v) == MerkleTree) then
            if not_quote then
                table.insert(quoted_args, v:hex_string())
            else
                table.insert(quoted_args, '"' .. v:hex_string() .. '"')
            end
        elseif type(v) == "table" then
            if v._tag == "tuple" then
                local qa = quote_args(v, true)
                local ca = table.concat(qa, ",")
                local sb = "'(" .. ca .. ")'"
                table.insert(quoted_args, sb)
            else
                local qa = quote_args(v, true)
                local ca = table.concat(qa, ",")
                local sb = "'[" .. ca .. "]'"
                table.insert(quoted_args, sb)
            end
        elseif not_quote then
            table.insert(quoted_args, tostring(v))
        else
            table.insert(quoted_args, '"' .. v .. '"')
        end
    end

    return quoted_args
end


local Client = {}
Client.__index = Client

function Client:new(account_index)
    local blockchain_data = require "blockchain.constants"

    local client = {
        endpoint = blockchain_data.endpoint,
        pk = blockchain_data.pks[account_index],
    }

    setmetatable(client, self)
    return client
end

local cast_logs_template = [==[
cast rpc -r "%s" eth_getLogs \
    '[{"fromBlock": "earliest", "toBlock": "latest", "address": "%s", "topics": [%s]}]' -w  2>&1
]==]

function Client:_read_logs(tournament_address, sig, topics, data_sig)
    topics = topics or { false, false, false }
    local encoded_sig = eth_ebi.encode_sig(sig)
    table.insert(topics, 1, encoded_sig)
    assert(#topics == 4, "topics doesn't have four elements")

    local topics_strs = {}
    for _, v in ipairs(topics) do
        local s
        if v then
            s = '"' .. v .. '"'
        else
            s = "null"
        end
        table.insert(topics_strs, s)
    end
    local topic_str = table.concat(topics_strs, ", ")

    local cmd = string.format(
        cast_logs_template,
        self.endpoint,
        tournament_address,
        topic_str
    )

    local handle = io.popen(cmd)
    assert(handle)
    local logs = handle:read "*a"
    handle:close()

    if logs:find "Error" then
        error(string.format("Read logs `%s` failed:\n%s", sig, logs))
    end

    local ret = parse_logs(logs, data_sig)
    return ret
end

local cast_call_template = [==[
cast call --rpc-url "%s" "%s" "%s" %s 2>&1
]==]

function Client:_call(address, sig, args)
    local quoted_args = {}
    for _, v in ipairs(args) do
        table.insert(quoted_args, '"' .. v .. '"')
    end
    local args_str = table.concat(quoted_args, " ")

    local cmd = string.format(
        cast_call_template,
        self.endpoint,
        address,
        sig,
        args_str
    )

    local handle = io.popen(cmd)
    assert(handle)

    local ret = {}
    local str = handle:read()
    while str do
        if str:find "Error" or str:find "error" then
            local err_str = handle:read "*a"
            handle:close()
            error(string.format("Call `%s` failed:\n%s%s", sig, str, err_str))
        end

        table.insert(ret, str)
        str = handle:read()
    end
    handle:close()

    return ret
end

function Client:read_match_created(tournament_address, commitment_hash)
    local sig = "matchCreated(bytes32,bytes32,bytes32)"
    local data_sig = "(bytes32)"

    local logs1 = self:_read_logs(tournament_address, sig, { commitment_hash:hex_string(), false, false }, data_sig)
    local logs2 = self:_read_logs(tournament_address, sig, { false, commitment_hash:hex_string(), false }, data_sig)

    local logs = sort_and_dedup(join_tables(logs1, logs2))

    local ret = {}
    for k, v in ipairs(logs) do
        local log = {}
        log.tournament_address = tournament_address
        log.meta = v.meta

        log.commitment_one = Hash:from_digest_hex(v.emited_topics[2])
        log.commitment_two = Hash:from_digest_hex(v.emited_topics[3])
        log.left_hash = Hash:from_digest_hex(v.decoded_data[1])
        log.match_id_hash = log.commitment_one:join(log.commitment_two)

        ret[k] = log
    end

    return ret
end

function Client:read_commitment(tournament_address, commitment_hash)
    local sig = "getCommitment(bytes32)((uint64,uint64),bytes32)"

    local call_ret = self:_call(tournament_address, sig, { commitment_hash:hex_string() })
    assert(#call_ret == 2)

    local allowance, last_resume = call_ret[1]:match "%((%d+),(%d+)%)"
    assert(allowance)
    assert(last_resume)
    local clock = {
        allowance = tonumber(allowance),
        last_resume = tonumber(last_resume)
    }

    local ret = {
        clock = clock,
        final_state = Hash:from_digest_hex(call_ret[2])
    }

    return ret
end

function Client:read_tournament_created(tournament_address, match_id_hash)
    local sig = "newInnerTournament(bytes32,address)"
    local data_sig = "(address)"

    local logs = self:_read_logs(tournament_address, sig, { match_id_hash:hex_string(), false, false }, data_sig)
    assert(#logs <= 1)

    if #logs == 0 then return false end
    local log = logs[1]

    local ret = {
        parent_match = match_id_hash,
        new_tournament = log.decoded_data[1],
    }

    return ret
end

function Client:match(address, match_id_hash)
    local sig = "getMatch(bytes32)(bytes32,bytes32,bytes32,uint256,uint64,uint64)"
    local ret = self:_call(address, sig, { match_id_hash:hex_string() })
    ret[1] = Hash:from_digest_hex(ret[1])
    ret[2] = Hash:from_digest_hex(ret[2])
    ret[3] = Hash:from_digest_hex(ret[3])

    return ret
end

function Client:inner_tournament_winner(address)
    local sig = "innerTournamentWinner()(bool,bytes32)"
    local ret = self:_call(address, sig, {})
    ret[2] = Hash:from_digest_hex(ret[2])

    return ret
end

function Client:root_tournament_winner(address)
    local sig = "arbitrationResult()(bool,bytes32,bytes32)"
    local ret = self:_call(address, sig, {})
    ret[2] = Hash:from_digest_hex(ret[2])
    ret[3] = Hash:from_digest_hex(ret[3])

    return ret
end

function Client:maximum_delay(address)
    local sig = "maximumEnforceableDelay()(uint64)"
    local ret = self:_call(address, sig, {})

    return ret
end

local cast_send_template = [[
cast send --private-key "%s" --rpc-url "%s" "%s" "%s" %s 2>&1
]]

function Client:_send_tx(tournament_address, sig, args)
    local quoted_args = quote_args(args)
    local args_str = table.concat(quoted_args, " ")

    local cmd = string.format(
        cast_send_template,
        self.pk,
        self.endpoint,
        tournament_address,
        sig,
        args_str
    )

    local handle = io.popen(cmd)
    assert(handle)

    local ret = handle:read "*a"
    if ret:find "Error" then
        handle:close()
        error(string.format("Send transaction `%s` reverted:\n%s", sig, ret))
    end
    handle:close()
end

function Client:tx_join_tournament(tournament_address, final_state, proof, left_child, right_child)
    local sig = [[joinTournament(bytes32,bytes32[],bytes32,bytes32)]]
    self:_send_tx(tournament_address, sig, { final_state, proof, left_child, right_child })
end

function Client:tx_advance_match(
    tournament_address, commitment_one, commitment_two, left, right, new_left, new_right
)
    local sig = [[advanceMatch((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32)]]
    self:_send_tx(
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, new_left, new_right }
    )
end

function Client:tx_seal_inner_match(
    tournament_address, commitment_one, commitment_two, left, right, initial_hash, proof
)
    local sig =
    [[sealInnerMatchAndCreateInnerTournament((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32[])]]
    self:_send_tx(
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, initial_hash:hex_string(), proof }
    )
end

function Client:tx_win_inner_match(tournament_address, child_tournament_address, left, right)
    local sig =
    [[winInnerMatch(address,bytes32,bytes32)]]
    self:_send_tx(
        tournament_address,
        sig,
        { child_tournament_address, left, right }
    )
end

function Client:tx_seal_leaf_match(
    tournament_address, commitment_one, commitment_two, left, right, initial_hash, proof
)
    local sig =
    [[sealLeafMatch((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32[])]]
    self:_send_tx(
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, initial_hash, proof }
    )
end

function Client:tx_win_leaf_match(
    tournament_address, commitment_one, commitment_two, left, right, proof
)
    local sig =
    [[winLeafMatch((bytes32,bytes32),bytes32,bytes32,bytes)]]
    self:_send_tx(
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, proof }
    )
end

local cast_advance_template = [[
cast rpc -r "%s" evm_increaseTime %d
]]

function Client:advance_time(seconds)
    local cmd = string.format(
        cast_advance_template,
        self.endpoint,
        seconds
    )

    local handle = io.popen(cmd)
    assert(handle)
    local ret = handle:read "*a"
    handle:close()

    if ret:find "Error" then
        error(string.format("Advance time `%d`s failed:\n%s", seconds, ret))
    end
end

return Client
