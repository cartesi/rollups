// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./Arithmetic.sol";

library GameClockLib {
    enum Turn {Challenger, Claimer}

    struct Timer {
        uint256 lastResume;
        uint256 challengerAllowance;
        uint256 claimerAllowance;
        Turn turn;
    }

    function newTimerChallengerTurn(
        uint256 currentTime,
        uint256 allowance
    )
        external
        pure
        returns(Timer memory)
    {
        return Timer(currentTime, allowance, allowance, Turn.Challenger);
    }

    function newTimerClaimerTurn(
        uint256 currentTime,
        uint256 allowance
    )
        external
        pure
        returns(Timer memory)
    {
        return Timer(currentTime, allowance, allowance, Turn.Claimer);
    }

    function challengerPassTurn(
        Timer memory timer,
        uint256 currentTime
    )
        external
        pure
        returns(Timer memory)
    {
        require(timer.turn == Turn.Challenger, "Not Challenger's turn");
        uint256 newAllowance = challengerAllowance(
            timer,
            currentTime
        );
        require(newAllowance != 0, "clock has no time left");

        return Timer(
            currentTime,
            newAllowance,
            timer.claimerAllowance,
            Turn.Claimer
        );
    }

    function claimerPassTurn(
        Timer memory timer,
        uint256 currentTime
    )
        external
        pure
        returns(Timer memory)
    {
        require(timer.turn == Turn.Claimer, "Not Claimer's turn");
        uint256 newAllowance = claimerAllowance(
            timer,
            currentTime
        );
        require(newAllowance != 0, "clock has no time left");

        return Timer(
            currentTime,
            timer.challengerAllowance,
            newAllowance,
            Turn.Challenger
        );
    }

    function challengerAllowance(
        Timer memory timer,
        uint256 currentTime
    )
        public
        pure
        returns(uint256)
    {
        if (timer.turn == Turn.Challenger) {
            return timer.challengerAllowance;
        }

        return computeAllowance(
            timer.lastResume,
            currentTime,
            timer.challengerAllowance
        );
    }

    function claimerAllowance(
        Timer memory timer,
        uint256 currentTime
    )
        public
        pure
        returns(uint256)
    {
        if (timer.turn == Turn.Challenger) {
            return timer.claimerAllowance;
        }

        return computeAllowance(
            timer.lastResume,
            currentTime,
            timer.claimerAllowance
        );
    }

    function challengerHasTimeLeft(
        Timer memory timer,
        uint256 currentTime
    )
        external
        pure
        returns(bool)
    {
        return challengerAllowance(
            timer,
            currentTime
        ) != 0;
    }

    function claimerHasTimeLeft(
        Timer memory timer,
        uint256 currentTime
    )
        external
        pure
        returns(bool)
    {
        return claimerAllowance(
            timer,
            currentTime
        ) != 0;
    }



    //
    // Internals
    //

    function computeAllowance(
        uint256 lastResume,
        uint256 currentTime,
        uint256 currentAllowance
    )
        internal
        pure
        returns(uint256)
    {
        assert(currentTime >= lastResume);
        uint256 timeElapsed = currentTime - lastResume;
        return Arithmetic.monus(currentAllowance, timeElapsed);
    }
}
