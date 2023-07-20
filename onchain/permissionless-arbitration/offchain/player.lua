local constants = require "constants"

local Player = {}
Player.__index = Player

function Player:new(root_tournament_address, client, machine)
    local player = {
        root_tournament = {
            address = root_tournament_address,
            level = constants.levels,
            parent = false,
        },
        client = client,
        machine = machine,
        commitments = {},
        called_win = {}
    }

    setmetatable(player, self)
    return player
end

function Player:react()
    if self.has_lost then return end
    return self:_react_tournament(self.root_tournament)
end

function Player:_react_tournament(tournament)
    local commitment = self.commitments[tournament.address]
    if not commitment then
        commitment = self.machine:commitment(
            constants.log2step[tournament.level],
            constants.heights[constants.levels - tournament.level + 1],
            false, -- TODO
            false -- TODO
        )
        self.commitments[tournament.address] = commitment
    end

    if not tournament.parent then
        local winner_final_state = self.client:root_tournament_winner(tournament.address)
        if winner_final_state[1] == "true" then
            print "TOURNAMENT FINISHED, HURRAYYY"
            print("Final state: " .. winner_final_state[2])
            return true
        end
    else
        local tournament_winner = self.client:tournament_winner(tournament.address)
        if not tournament_winner:is_zero() then
            local old_commitment = self.commitments[tournament.parent.address]
            if tournament_winner ~= old_commitment.root then
                print "player lost tournament"
                self.has_lost = true
                return
            end

            if self.called_win[tournament.address] then
                print "player already called winInnerMatch"
                return
            else
                self.called_win[tournament.address] = true
            end

            print(string.format(
                "win tournament %s of level %d for commitment %s",
                tournament.address,
                tournament.level,
                commitment.root
            ))
            local _, left, right = old_commitment:children(old_commitment.root)
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

    -- print("HEIGHT", match.current_height)
    if match.current_height == 0 then
        -- match sealed
        if match.tournament.level == 1 then
            local f, left, right = commitment:children(commitment.root)
            assert(f)

            local finished =
                self.client:match(match.tournament.address, match.match_id_hash)[1]:is_zero()

            if finished then
                local delay = tonumber(self.client:maximum_delay(match.tournament.address)[1])
                print("DELAY", delay - os.time())
                return
            end

            print(string.format(
                "win leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root
            ))
            self.client:tx_win_leaf_match(
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right
            )

        else
            local address = self.client:read_tournament_created(
                match.tournament.address,
                match.match_id_hash
            ).new_tournament

            local new_tournament = {}
            new_tournament.address = address
            new_tournament.level = match.tournament.level - 1
            new_tournament.parent = match.tournament

            return self:_react_tournament(new_tournament)
        end

    elseif match.current_height == 1 then
        -- match to be sealed
        local found, left, right = commitment:children(match.current_other_parent)
        if not found then return end

        local initial_hash, proof
        if match.running_leaf == 0 then
            initial_hash, proof = self.machine.initial_hash, {}
        else
            initial_hash, proof = commitment:prove_leaf(match.running_leaf)
        end

        if match.tournament.level == 1 then
            print(string.format(
                "seal leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root
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
            print(string.format(
                "seal inner match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root
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

            return self:_react_tournament(new_tournament)
        end
    else
        -- match running
        local found, left, right = commitment:children(match.current_other_parent.digest_hex)
        if not found then return end

        local new_left, new_right
        if left ~= match.current_left then
            local f
            f, new_left, new_right = commitment:children(left)
            assert(f)
        else
            local f
            f, new_left, new_right = commitment:children(right)
            assert(f)
        end

        print(string.format(
            "advance match with current height %d in tournament %s of level %d for commitment %s",
            match.current_height,
            match.tournament.address,
            match.tournament.level,
            commitment.root
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
    local matches = self.client:read_match_created(tournament.address, commitment.root)
    local last_match = matches[#matches]

    if not last_match then return false end

    local m = self.client:match(tournament.address, last_match.match_id_hash)
    if m[1]:is_zero() then return false end
    last_match.current_other_parent = m[1]
    last_match.current_left = m[2]
    last_match.running_leaf = tonumber(m[4])
    last_match.height = tonumber(m[5])
    last_match.current_height = tonumber(m[6])
    last_match.tournament = tournament

    return last_match
end

function Player:_join_tournament_if_needed(tournament, commitment)
    local c = self.client:read_commitment(tournament.address, commitment.root)

    if c.clock.allowance == 0 then
        local f, left, right = commitment:children(commitment.root)
        assert(f)
        local last, proof = commitment:last()

        print(string.format(
            "join tournament %s of level %d with commitment %s",
            tournament.address,
            tournament.level,
            commitment.root
        ))
        self.client:tx_join_tournament(tournament.address, last, proof, left, right)
    end
end

return Player
