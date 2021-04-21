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
} from "@ethereum-waffle/mock-contract";
import { solidity } from 'ethereum-waffle'
import { InputImpl__factory } from '../src/types/factories/InputImpl__factory'
import { Signer } from 'ethers'
import { InputImpl } from '../src/types/InputImpl'

use(solidity)

describe('Input Implementation', async () => {
  let signer: Signer;
  let inputImpl: InputImpl;
  let mockDescartesv2: MockContract; //mock descartesv2 implementation

  const log2Size = 7;
  
  beforeEach(async () => {
    [signer] = await ethers.getSigners();

    const DescartesV2 = await deployments.getArtifact("DescartesV2");

    mockDescartesv2 = await deployMockContract(signer, DescartesV2.abi);

    const inputFactory = new InputImpl__factory(signer);

    inputImpl = await inputFactory.deploy(mockDescartesv2.address, log2Size)

  })

  it('addInput should revert if input length == 0', async () => {
    await expect(
      inputImpl.addInput([]),
      'empty input should revert',
    ).to.be.revertedWith('input is empty')
  })

  it('addInput should revert if input is larger than drive (log2Size)', async () => {
    var input_150_bytes = Buffer.from("a".repeat(150), "utf-8");
    // one extra byte
    var input_129_bytes = Buffer.from("a".repeat(129), "utf-8");

    await expect(
      inputImpl.addInput(input_150_bytes),
      'input cant be bigger than drive',
    ).to.be.revertedWith('input is larger than drive')

    // input shouldnt fit because of one byte
    await expect(
      inputImpl.addInput(input_129_bytes),
      'input should still revert because metadata doesnt fit',
    ).to.be.revertedWith('input is larger than drive')
  })

  it('addInput should add input to inbox', async () => {
    var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");
    
    mockDescartesv2.mock.notifyInput.returns(false);
    mockDescartesv2.mock.notifyInput.returns(false);
    mockDescartesv2.mock.notifyInput.returns(false);

    await inputImpl.addInput(input_64_bytes)
    await inputImpl.addInput(input_64_bytes)
    await inputImpl.addInput(input_64_bytes)

    expect(
      await inputImpl.getNumberOfInputs(),
      "Number of inputs should be zero, because non active inbox is empty"
    ).to.equal(0);
    
    mockDescartesv2.mock.notifyInput.returns(true);

    await inputImpl.addInput(input_64_bytes);

    expect(
      await inputImpl.getNumberOfInputs(),
      "Number of inputs should be 3, because last addition changes the inbox"
    ).to.equal(3);
  });

  it('getNumberOfInputs should return from correct inbox', async () => {
    var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");
    
    mockDescartesv2.mock.notifyInput.returns(false);
    //mockDescartesv2.mock.notifyInput.returns(false);
    //mockDescartesv2.mock.notifyInput.returns(false);

    await inputImpl.addInput(input_64_bytes);
    //await inputImpl.addInput(input_64_bytes);
    //await inputImpl.addInput(input_64_bytes);

    expect(
      await inputImpl.getNumberOfInputs(),
      "previous inbox should return zero"
    ).to.equal(0);

    //mockDescartesv2.mock.notifyInput.returns(true);
    //await inputImpl.addInput(input_64_bytes);

    //expect(
    //  await inputImpl.getNumberOfInputs(),
    //  "non active inbox should have 3 inputs"
    //).to.equal(3);

    //mockDescartesv2.mock.notifyInput.returns(true);
    //await inputImpl.addInput(input_64_bytes);

    //expect(
    //  await inputImpl.getNumberOfInputs(),
    //  "non active inbox should have 1 input"
    //).to.equal(1);
  });

  it('onNewEpoch() can only be called by descartesv2', async () => {
    await expect(
      inputImpl.onNewEpoch(),
      'function can only be called by descartesv2',
    ).to.be.revertedWith('Only descartesV2 can call this function')
  });

  it('onNewInputAccumulation() can only be called by descartesv2', async () => {
    await expect(
      inputImpl.onNewInputAccumulation(),
      'function can only be called by descartesv2',
    ).to.be.revertedWith('Only descartesV2 can call this function')
  });

  it('getCurrentInbox should return correct inbox', async () => {
    var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

    mockDescartesv2.mock.notifyInput.returns(true);
    expect(
      await inputImpl.getCurrentInbox(),
      "current inbox should start as zero"
    ).to.equal(0);

    mockDescartesv2.mock.notifyInput.returns(false);
    await inputImpl.addInput(input_64_bytes);

    expect(
      await inputImpl.getCurrentInbox(),
      "inbox shouldnt change if notifyInput returns false"
    ).to.equal(0);

    mockDescartesv2.mock.notifyInput.returns(true);
    await inputImpl.addInput(input_64_bytes);

    expect(
      await inputImpl.getCurrentInbox(),
      "inbox should change if notifyInput returns true"
    ).to.equal(1);

    mockDescartesv2.mock.notifyInput.returns(false);
    await inputImpl.addInput(input_64_bytes);

    expect(
      await inputImpl.getCurrentInbox(),
      "inbox shouldnt change if notifyInput returns false (2)"
    ).to.equal(1);

  });
})
