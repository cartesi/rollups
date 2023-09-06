local constants = require "constants"
local bint = require 'utils.bint' (256) -- use 256 bits integers

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
            commitment = false,
            commitment_status = {},
            tournament_winner = {},
            latest_match = {},
        },
        client = Client:new(player_index),
        commitment_builder = commitment_builder,
        player_index = player_index
    }

    setmetatable(player, self)
    return player
end

function Player:fetch()
    return self:_fetch_tournament(self.root_tournament)
end

function Player:_fetch_tournament(tournament)
    local commitment = tournament.commitment
    if not commitment then
        commitment = self.commitment_builder:build(
            tournament.base_big_cycle,
            tournament.level
        )
        tournament.commitment = commitment
    end

    if not tournament.parent then
        tournament.tournament_winner = self.client:root_tournament_winner(tournament.address)
    else
        tournament.tournament_winner = self.client:inner_tournament_winner(tournament.address)
    end

    tournament.latest_match =  self:_latest_match(tournament)

    if not tournament.latest_match then
        tournament.commitment_status = self.client:read_commitment(tournament.address, commitment.root_hash)
    else
        self:_fetch_match(tournament.latest_match, commitment)
    end
end

function Player:_fetch_match(match, commitment)
    if match.current_height == 0 then
        -- match sealed
        if match.tournament.level == 1 then
            local f, left, right = commitment.root_hash:children()
            assert(f)

            match.finished =
                self.client:match(match.tournament.address, match.match_id_hash)[1]:is_zero()

            if match.finished then
                match.delay = tonumber(self.client:maximum_delay(match.tournament.address)[1])
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
            match.inner_tournament = new_tournament

            return self:_fetch_tournament(new_tournament)
        end
    end
end

function Player:_latest_match(tournament)
    local commitment = tournament.commitment
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

return Player
