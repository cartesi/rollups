// Copyright (C) 2020 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import { expect, use } from 'chai'
import { deployments, ethers } from 'hardhat'
import {
  deployMockContract,
  MockContract,
} from '@ethereum-waffle/mock-contract'
import { solidity, MockProvider } from 'ethereum-waffle'
import { ValidatorManagerImpl__factory } from '../src/types/factories/ValidatorManagerImpl__factory'
import { BytesLike, Signer } from 'ethers'
import { ValidatorManagerImpl } from '../src/types/ValidatorManagerImpl'
import { AnyARecord } from 'dns'
import { isBytesLike } from 'ethers/lib/utils'

use(solidity)

describe('Validator Manager Implementation', async () => {
  var descartesV2: Signer
  var VMI: ValidatorManagerImpl
  const provider = new MockProvider()
  var validators: string[] = []

  let hash_zero = ethers.constants.HashZero
  let address_zero = '0x0000000000000000000000000000000000000000'

  enum Result {
    NoConflict,
    Consensus,
    Conflict,
  }

  beforeEach(async () => {
    [descartesV2] = await ethers.getSigners()
    const vmiFactory = new ValidatorManagerImpl__factory(descartesV2)
    var address: any

    var wallets = provider.getWallets()
    validators = []

    // add all wallets as validators
    for (var wallet of wallets) {
      address = await wallet.getAddress()
      validators.push(address)
    }

    VMI = await vmiFactory.deploy(await descartesV2.getAddress(), validators)
  })

  it('onClaim should revert if claim is 0x00', async () => {
    await expect(
      VMI.onClaim(validators[0], hash_zero),
      'should revert if claim == 0x00',
    ).to.be.revertedWith('claim cannot be 0x00')
  })

  it('onClaim NoConflict and Consensus', async () => {
    var claim = '0x' + '1'.repeat(64);

    // if validators keep agreeing there is no conflict
    for (var i = 0; i < validators.length - 1; i++) {
      await expect(
        await VMI.onClaim(validators[i], claim),
        'equal claims should not generate conflict nor consensus, if not all validators have agreed',
      )
        .to.emit(VMI, 'ClaimReceived')
        .withArgs(
          Result.NoConflict,
          [hash_zero, hash_zero],
          [address_zero, address_zero],
        )
    }
    // when last validator agrees, should return consensus
    var lastValidator = validators[validators.length - 1]
    await expect(
      await VMI.onClaim(lastValidator, claim),
      'after all validators claim should be consensus',
    )
      .to.emit(VMI, 'ClaimReceived')
      .withArgs(
        Result.Consensus,
        [claim, hash_zero],
        [lastValidator, address_zero],
      )
  })

  it('onClaim with different claims should return conflict', async () => {
    var claim = '0x' + '1'.repeat(64)
    var claim2 = '0x' + '2'.repeat(64)

    await expect(
      await VMI.onClaim(validators[0], claim),
      'first claim should not generate conflict',
    )
      .to.emit(VMI, 'ClaimReceived')
      .withArgs(
        Result.NoConflict,
        [hash_zero, hash_zero],
        [address_zero, address_zero],
      )

    await expect(
      await VMI.onClaim(validators[1], claim2),
      'different claim should generate conflict',
    )
      .to.emit(VMI, 'ClaimReceived')
      .withArgs(
        Result.Conflict,
        [claim, claim2],
        [validators[0], validators[1]],
      )
  })

  it('onDisputeEnd with no conflict after', async () => {
    var claim = '0x' + '1'.repeat(64)

    // start with no conflict claim to populate
    // variables
    await VMI.onClaim(validators[0], claim);

    await expect(
        await VMI.onDisputeEnd(validators[0], validators[1], claim),
        "if winning claim is current claim and there is no consensus, should return NoConflict",
    )
        .to.emit(VMI, "DisputeEnded")
        .withArgs(
            Result.NoConflict,
            [],
            []
        )

  })

  it('onDisputeEnd with consensus after', async () => {
    var claim = '0x' + '1'.repeat(64)
    var lastValidator = validators[validators.length - 1]

    // all validators agree but last one
    for (var i = 0; i < validators.length - 1; i++) {
        await VMI.onClaim(validators[i], claim);
    }

    // last validator lost dispute, the only one that disagreed
    await expect(
        await VMI.onDisputeEnd(validators[0], lastValidator, claim),
        "if losing claim was the only one not agreeing, should return consensus",
    )
        .to.emit(VMI, "DisputeEnded")
        .withArgs(
            Result.Consensus,
            [claim, hash_zero],
            [validators[0], address_zero]
        )
  });

  it('onDisputeEnd multiple validators defending lost claim', async () => {
    var claim = '0x' + '1'.repeat(64)
    var claim2 = '0x' + '2'.repeat(64)
    var lastValidator = validators[validators.length - 1]

    // all validators agree but last one
    for (var i = 0; i < validators.length - 1; i++) {
        await VMI.onClaim(validators[i], claim);
    }
    // last validator lost dispute, the only one that disagreed
    // next defender should be validators[1]
    await expect(
        await VMI.onDisputeEnd(lastValidator, validators[0], claim2),
        "conflict should continue if there are validators still defending claim that lost",
    )
        .to.emit(VMI, "DisputeEnded")
        .withArgs(
            Result.Conflict,
            [claim, claim2],
            [validators[1], lastValidator]
        )

    // make all other validators but last agreeing one lose dispute
    for (var i = 1; i < validators.length - 2; i++) {
        await VMI.onDisputeEnd(lastValidator, validators[i], claim2);
    }

    // honest validator by himself can generate consensus
    // by winning his last dispute
    await expect(
        await VMI.onDisputeEnd(lastValidator, validators[validators.length - 2], claim2),
        "lastValidator should be the last one in the validator set",
    )
        .to.emit(VMI, "DisputeEnded")
        .withArgs(
            Result.Consensus,
            [claim2, hash_zero],
            [lastValidator, address_zero]
        )
  });

  it('onNewEpoch', async () => {
    var claim = '0x' + '1'.repeat(64);

    // one validator claims
    await VMI.onClaim(validators[0], claim);

    // epoch ends without consensus
    await expect(
        await VMI.onNewEpoch(),
        "new epoch should return current claim"
    ).to.emit(VMI, "NewEpoch").withArgs(claim);

    expect(
        await VMI.getCurrentAgreementMask(),
        "current agreement mask should reset"
    ).to.equal(0)

    expect(
        await VMI.getCurrentClaim(),
        "current claim should reset"
    ).to.equal(hash_zero)

  })
})
