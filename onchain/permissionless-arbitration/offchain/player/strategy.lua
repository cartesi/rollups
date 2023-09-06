local constants = require "constants"
local helper = require 'utils.helper'

local Machine = require "computation.machine"

local _react_match_honestly
local _react_tournament_honestly

local function _join_tournament_if_needed(player, tournament)
    if tournament.commitment_status.clock.allowance == 0 then
        local f, left, right = tournament.commitment:children(tournament.commitment.root_hash)
        assert(f)
        local last, proof = tournament.commitment:last()

        helper.log(player.player_index, string.format(
            "join tournament %s of level %d with commitment %s",
            tournament.address,
            tournament.level,
            tournament.commitment.root_hash
        ))
        player.client:tx_join_tournament(tournament.address, last, proof, left, right)
    else
        helper.touch_player_idle(player.player_index)
    end
end

_react_match_honestly = function(player, match, commitment)
    -- TODO call timeout if needed

    helper.log(player.player_index, "HEIGHT: " .. match.current_height)
    if match.current_height == 0 then
        -- match sealed
        if match.tournament.level == 1 then
            local f, left, right = commitment.root_hash:children()
            assert(f)

            helper.log(player.player_index, string.format(
                "Calculating access logs for step %s",
                match.running_leaf
            ))

            local cycle = (match.running_leaf >> constants.log2_uarch_span):touinteger()
            local ucycle = (match.running_leaf & constants.uarch_span):touinteger()
            local logs = Machine:get_logs(player.machine_path, cycle, ucycle)

            helper.log(player.player_index, string.format(
                "win leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            local ok, e = pcall(player.client.tx_win_leaf_match,
                player.client,
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                logs
            )
            if not ok then
                helper.log(player.player_index, string.format(
                    "win leaf match reverted: %s",
                    e
                ))
            end
        else
            return _react_tournament_honestly(player, match.inner_tournament)
        end
    elseif match.current_height == 1 then
        -- match to be sealed
        local found, left, right = match.current_other_parent:children()
        if not found then
            helper.touch_player_idle(player.player_index)
            return
        end

        local initial_hash, proof
        if match.running_leaf:iszero() then
            initial_hash, proof = commitment.implicit_hash, {}
        else
            initial_hash, proof = commitment:prove_leaf(match.running_leaf)
        end

        if match.tournament.level == 1 then
            helper.log(player.player_index, string.format(
                "seal leaf match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            player.client:tx_seal_leaf_match(
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                initial_hash,
                proof
            )
        else
            helper.log(player.player_index, string.format(
                "seal inner match in tournament %s of level %d for commitment %s",
                match.tournament.address,
                match.tournament.level,
                commitment.root_hash
            ))
            player.client:tx_seal_inner_match(
                match.tournament.address,
                match.commitment_one,
                match.commitment_two,
                left,
                right,
                initial_hash,
                proof
            )
        end
    else
        -- match running
        local found, left, right = match.current_other_parent:children()
        if not found then
            helper.touch_player_idle(player.player_index)
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

        helper.log(player.player_index, string.format(
            "advance match with current height %d in tournament %s of level %d for commitment %s",
            match.current_height,
            match.tournament.address,
            match.tournament.level,
            commitment.root_hash
        ))
        player.client:tx_advance_match(
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

_react_tournament_honestly = function(player, tournament)
    local tournament_winner = tournament.tournament_winner
    if tournament_winner[1] == "true" then
        if not tournament.parent then
            helper.log(player.player_index, "TOURNAMENT FINISHED, HURRAYYY")
            helper.log(player.player_index, "Winner commitment: " .. tournament_winner[2]:hex_string())
            helper.log(player.player_index, "Final state: " .. tournament_winner[3]:hex_string())
            return true
        else
            local old_commitment = tournament.parent.commitment
            if tournament_winner[2] ~= old_commitment.root_hash then
                helper.log(player.player_index, "player lost tournament")
                player.has_lost = true
                return
            end

            if tournament.called_win then
                helper.log(player.player_index, "player already called winInnerMatch")
                return
            else
                tournament.called_win = true
            end

            helper.log(player.player_index, string.format(
                "win tournament %s of level %d for commitment %s",
                tournament.address,
                tournament.level,
                tournament.commitment.root_hash
            ))
            local _, left, right = old_commitment:children(old_commitment.root_hash)
            player.client:tx_win_inner_match(tournament.parent.address, tournament.address, left, right)
            return
        end
    end

    if not tournament.latest_match then
        _join_tournament_if_needed(player, tournament)
    else
        _react_match_honestly(player, tournament.latest_match, tournament.commitment)
    end
end

local function _react_honestly(player)
    if player.has_lost then
        return true
    end
    return _react_tournament_honestly(player, player.root_tournament)
end

return {
    react_honestly = _react_honestly
}
