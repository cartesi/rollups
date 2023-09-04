local constants = require "constants"
local bint = require 'utils.bint' (256) -- use 256 bits integers
local helper = require 'utils.helper'

local Machine = require "computation.machine"
local Client = require "blockchain.client"

local Player = {}
Player.__index = Player

function Player:new(root_tournament_address, player_index, commitment_builder, machine_path)
    local player = {
        machine_path = machine_path,
        root_tournament = {
            base_big_cycle = 0,
            address = root_tournament_address,
            level = constants.levels,
            parent = false,
        },
        client = Client:new(player_index),
        commitment_builder = commitment_builder,
        commitments = {},
        called_win = {},
        player_index = player_index
    }

    setmetatable(player, self)
    return player
end

function Player:react()
    if self.has_lost then
        return true
    end
    return self:_react_tournament(self.root_tournament)
end

function Player:_react_tournament(tournament)
    local commitment = self.commitments[tournament.address]
    if not commitment then
        commitment = self.commitment_builder:build(
            tournament.base_big_cycle,
            tournament.level
        )
        self.commitments[tournament.address] = commitment
    end

    if not tournament.parent then
        local winner_final_state = self.client:root_tournament_winner(tournament.address)
        if winner_final_state[1] == "true" then
            helper.log(self.player_index, "TOURNAMENT FINISHED, HURRAYYY")
            helper.log(self.player_index, "Winner commitment: " .. winner_final_state[2]:hex_string())
            helper.log(self.player_index, "Final state: " .. winner_final_state[3]:hex_string())
            return true
        end
    else
        local tournament_winner = self.client:inner_tournament_winner(tournament.address)
        if tournament_winner[1] == "true" then
            local old_commitment = self.commitments[tournament.parent.address]
            if tournament_winner[2] ~= old_commitment.root_hash then
                helper.log(self.player_index, "player lost tournament")
                self.has_lost = true
                return
            end

            if self.called_win[tournament.address] then
                helper.log(self.player_index, "player already called winInnerMatch")
                return
            else
                self.called_win[tournament.address] = true
            end

            helper.log(self.player_index, string.format(
                "win tournament %s of level %d for commitment %s",
                tournament.address,
                tournament.level,
                commitment.root_hash
            ))
            local _, left, right = old_commitment:children(old_commitment.root_hash)
            self.client:tx_win_inner_match(tournament.parent.address, tournament.address, left, right)
            return
        end
    end

    local latest_match = self:_latest_match(tournament, commitment)

    if not latest_match then
        self:_join_tournament_if_needed(tournament, commitment)
    else
        self:_react_match(latest_match, commitment)
    end
end

function Player:_react_match(match, commitment)
    -- TODO call timeout if needed

    helper.log(self.player_index, "HEIGHT: " .. match.current_height)
    if match.current_height == 0 then
        -- match sealed
        if match.tournament.level == 1 then
            local f, left, right = commitment.root_hash:children()
            assert(f)

            local finished =
                self.client:match(match.tournament.address, match.match_id_hash)[1]:is_zero()

            if finished then
                local delay = tonumber(self.client:maximum_delay(match.tournament.address)[1])
                helper.log(self.player_index, "DELAY", delay - os.time())
                return
            end

            helper.log(self.player_index, string.format(
                "Calculating access logs for step %s",
                match.running_leaf
            ))

            local cycle = (match.running_leaf >> constants.log2_uarch_span):touinteger()
            local ucycle = (match.running_leaf & constants.uarch_span):touinteger()
            local logs = Machine:get_logs(self.machine_path, cycle, ucycle)

            helper.log(self.player_index, string.format(
                "win leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            local ok, e = pcall(self.client.tx_win_leaf_match,
                self.client,
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                logs
            )
            if not ok then
                helper.log(self.player_index, string.format(
                    "win leaf match reverted: %s",
                    e
                ))
            end
        else
            local address = self.client:read_tournament_created(
                match.tournament.address,
                match.match_id_hash
            ).new_tournament

            local new_tournament = {}
            new_tournament.address = address
            new_tournament.level = match.tournament.level - 1
            new_tournament.parent = match.tournament
            new_tournament.base_big_cycle = match.base_big_cycle

            return self:_react_tournament(new_tournament)
        end
    elseif match.current_height == 1 then
        -- match to be sealed
        local found, left, right = match.current_other_parent:children()
        if not found then
            helper.touch_player_idle(self.player_index)
            return
        end

        local initial_hash, proof
        if match.running_leaf:iszero() then
            initial_hash, proof = commitment.implicit_hash, {}
        else
            initial_hash, proof = commitment:prove_leaf(match.running_leaf)
        end

        if match.tournament.level == 1 then
            helper.log(self.player_index, string.format(
                "seal leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            self.client:tx_seal_leaf_match(
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                initial_hash,
                proof
            )
        else
            helper.log(self.player_index, string.format(
                "seal inner match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            self.client:tx_seal_inner_match(
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                initial_hash,
                proof
            )

            local address = self.client:read_tournament_created(
                match.tournament.address,
                match.match_id_hash
            ).new_tournament

            local new_tournament = {}
            new_tournament.address = address
            new_tournament.level = match.tournament.level - 1
            new_tournament.parent = match.tournament
            new_tournament.base_big_cycle = match.base_big_cycle

            return self:_react_tournament(new_tournament)
        end
    else
        -- match running
        local found, left, right = match.current_other_parent:children()
        if not found then
            helper.touch_player_idle(self.player_index)
            return
        end

        local new_left, new_right
        if left ~= match.current_left then
            local f
            f, new_left, new_right = left:children()
            assert(f)
        else
            local f
            f, new_left, new_right = right:children()
            assert(f)
        end

        helper.log(self.player_index, string.format(
            "advance match with current height %d in tournament %s of level %d for commitment %s",
            match.current_height,
            match.tournament.address,
            match.tournament.level,
            commitment.root_hash
        ))
        self.client:tx_advance_match(
            match.tournament.address,
            match.commitment_one,
            match.commitment_two,
            left,
            right,
            new_left,
            new_right
        )
    end
end

function Player:_latest_match(tournament, commitment)
    local matches = self.client:read_match_created(tournament.address, commitment.root_hash)
    local last_match = matches[#matches]

    if not last_match then return false end

    local m = self.client:match(tournament.address, last_match.match_id_hash)
    if m[1]:is_zero() and m[2]:is_zero() and m[3]:is_zero() then
        return false
    end
    last_match.current_other_parent = m[1]
    last_match.current_left = m[2]
    last_match.current_right = m[3]
    last_match.running_leaf = bint(m[4])
    last_match.current_height = tonumber(m[5])
    last_match.level = tonumber(m[6])
    last_match.tournament = tournament

    local level = tournament.level
    local base = bint(tournament.base_big_cycle)
    local step = bint(1) << constants.log2step[level]
    last_match.leaf_cycle = base + (step * last_match.running_leaf)
    last_match.base_big_cycle = (last_match.leaf_cycle >> constants.log2_uarch_span):touinteger()

    return last_match
end

function Player:_join_tournament_if_needed(tournament, commitment)
    local c = self.client:read_commitment(tournament.address, commitment.root_hash)

    if c.clock.allowance == 0 then
        local f, left, right = commitment:children(commitment.root_hash)
        assert(f)
        local last, proof = commitment:last()

        helper.log(self.player_index, string.format(
            "join tournament %s of level %d with commitment %s",
            tournament.address,
            tournament.level,
            commitment.root_hash
        ))
        self.client:tx_join_tournament(tournament.address, last, proof, left, right)
    else
        helper.touch_player_idle(self.player_index)
    end
end

return Player
