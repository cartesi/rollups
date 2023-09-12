local Hash = require "cryptography.hash"
local MerkleTree = require "cryptography.merkle_tree"

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


local Sender = {}
Sender.__index = Sender

function Sender:new(account_index)
    local blockchain_data = require "blockchain.constants"

    local sender = {
        endpoint = blockchain_data.endpoint,
        pk = blockchain_data.pks[account_index],
        index = account_index,
        tx_count = 0
    }

    setmetatable(sender, self)
    return sender
end

local cast_send_template = [[
cast send --private-key "%s" --rpc-url "%s" "%s" "%s" %s 2>&1
]]

function Sender:_send_tx(tournament_address, sig, args)
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

    self.tx_count = self.tx_count + 1
    handle:close()
end

function Sender:tx_join_tournament(tournament_address, final_state, proof, left_child, right_child)
    local sig = [[joinTournament(bytes32,bytes32[],bytes32,bytes32)]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { final_state, proof, left_child, right_child }
    )
end

function Sender:tx_advance_match(
    tournament_address, commitment_one, commitment_two, left, right, new_left, new_right
)
    local sig = [[advanceMatch((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32)]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, new_left, new_right }
    )
end

function Sender:tx_seal_inner_match(
    tournament_address, commitment_one, commitment_two, left, right, initial_hash, proof
)
    local sig =
    [[sealInnerMatchAndCreateInnerTournament((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32[])]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, initial_hash:hex_string(), proof }
    )
end

function Sender:tx_win_inner_match(tournament_address, child_tournament_address, left, right)
    local sig =
    [[winInnerMatch(address,bytes32,bytes32)]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { child_tournament_address, left, right }
    )
end

function Sender:tx_seal_leaf_match(
    tournament_address, commitment_one, commitment_two, left, right, initial_hash, proof
)
    local sig =
    [[sealLeafMatch((bytes32,bytes32),bytes32,bytes32,bytes32,bytes32[])]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, initial_hash, proof }
    )
end

function Sender:tx_win_leaf_match(
    tournament_address, commitment_one, commitment_two, left, right, proof
)
    local sig =
    [[winLeafMatch((bytes32,bytes32),bytes32,bytes32,bytes)]]
    return pcall(
        self._send_tx,
        self,
        tournament_address,
        sig,
        { { commitment_one, commitment_two, _tag = "tuple" }, left, right, proof }
    )
end

return Sender
