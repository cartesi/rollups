local constants = require "constants"
local bint = require 'utils.bint' (256) -- use 256 bits integers

local Reader = require "blockchain.reader"

local State = {}
State.__index = State

function State:new(root_tournament_address)
    local state = {
        root_tournament = {
            base_big_cycle = 0,
            address = root_tournament_address,
            level = constants.levels,
            parent = false,
            commitments = {},
            matches = {},
            tournament_winner = {}
        },
        reader = Reader:new()
    }

    setmetatable(state, self)
    return state
end

function State:fetch()
    return self:_fetch_tournament(self.root_tournament)
end

function State:_fetch_tournament(tournament)
    local matches =  self:_matches(tournament)
    local commitments = self.reader:read_commitment_joined(tournament.address)

    for _, log in ipairs(commitments) do
        local root = log.root
        local status = self.reader:read_commitment(tournament.address, root)
        tournament.commitments[root] = { status = status, latest_match = false }
    end

    for _, match in ipairs(matches) do
        if match then
            self:_fetch_match(match)
            tournament.commitments[match.commitment_one].latest_match = match
            tournament.commitments[match.commitment_two].latest_match = match
        end
    end
    tournament.matches = matches

    if not tournament.parent then
        tournament.tournament_winner = self.reader:root_tournament_winner(tournament.address)
    else
        tournament.tournament_winner = self.reader:inner_tournament_winner(tournament.address)
    end
end

function State:_fetch_match(match)
    if match.current_height == 0 then
        -- match sealed
        if match.tournament.level == 1 then

            match.finished =
                self.reader:match(match.tournament.address, match.match_id_hash)[1]:is_zero()

            if match.finished then
                match.delay = tonumber(self.reader:maximum_delay(match.tournament.address)[1])
            end
        else
            local address = self.reader:read_tournament_created(
                match.tournament.address,
                match.match_id_hash
            ).new_tournament

            local new_tournament = {}
            new_tournament.address = address
            new_tournament.level = match.tournament.level - 1
            new_tournament.parent = match.tournament
            new_tournament.base_big_cycle = match.base_big_cycle
            new_tournament.commitments = {}
            match.inner_tournament = new_tournament

            return self:_fetch_tournament(new_tournament)
        end
    end
end

function State:_matches(tournament)
    local matches = self.reader:read_match_created(tournament.address)

    for k, match in ipairs(matches) do
        local m = self.reader:match(tournament.address, match.match_id_hash)
        if m[1]:is_zero() and m[2]:is_zero() and m[3]:is_zero() then
            matches[k] = false
        else
            match.current_other_parent = m[1]
            match.current_left = m[2]
            match.current_right = m[3]
            match.running_leaf = bint(m[4])
            match.current_height = tonumber(m[5])
            match.level = tonumber(m[6])
            match.tournament = tournament

            local level = tournament.level
            local base = bint(tournament.base_big_cycle)
            local step = bint(1) << constants.log2step[level]
            match.leaf_cycle = base + (step * match.running_leaf)
            match.base_big_cycle = (match.leaf_cycle >> constants.log2_uarch_span):touinteger()
        end
    end

    return matches
end

return State
