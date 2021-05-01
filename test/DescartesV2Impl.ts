import { deployments, ethers } from 'hardhat'
import { expect, use } from 'chai'
import { solidity } from 'ethereum-waffle';
import { Signer } from 'ethers'
import {
  deployMockContract,
  MockContract,
} from "@ethereum-waffle/mock-contract";
import { DescartesV2Impl } from '../src/types/DescartesV2Impl'
import { DescartesV2Impl__factory } from '../src/types/factories/DescartesV2Impl__factory'
import { formatBytes32String, parseBytes32String } from '@ethersproject/strings';
import { ProjectPathsUserConfig } from 'hardhat/types';

use(solidity)

///define a new function timeout to return a Promise after ms delay
let timeout =(ms)=> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("Descartes V2 Implementation", ()=> {
  ///mock a contract as if it is already implemented. Check Waffle for details
  let mockInput: MockContract;
  let mockOutput: MockContract;
  let mockValidatorManager: MockContract;
  let mockDisputeManager: MockContract;

  let descartesV2Impl: DescartesV2Impl;

  const inputDuration = 3;
  const challengePeriod = 3; 

  let signers: Signer[];

  ///each address is 20 bytes
  let address_zero = '0x0000000000000000000000000000000000000000'

  ///let enum starts from 0
  enum Phase {InputAccumulation=0, AwaitingConsensus=1, AwaitingDispute=2}; 
  enum Result {NoConflict=0, Consensus=1, Conflict=2}; 

  beforeEach(async () => {
    signers = await ethers.getSigners();

    const Input = await deployments.getArtifact("Input");
    const Output = await deployments.getArtifact("Output");
    const ValidatorManager = await deployments.getArtifact("ValidatorManager");
    const DisputeManager = await deployments.getArtifact("DisputeManager");

    mockInput = await deployMockContract(signers[0], Input.abi);
    mockOutput = await deployMockContract(signers[0], Output.abi);
    mockValidatorManager = await deployMockContract(signers[0], ValidatorManager.abi);
    mockDisputeManager = await deployMockContract(signers[0], DisputeManager.abi);

    const descartesV2ImplFactory = new DescartesV2Impl__factory(signers[0]);

    descartesV2Impl = await descartesV2ImplFactory.deploy(mockInput.address, mockOutput.address, mockValidatorManager.address, mockDisputeManager.address, inputDuration, challengePeriod);
    
  });
  
  ///***test function getCurrentPhase()***///
  it("initial phase should be InputAccumulation", async ()=>{
    expect(
      await descartesV2Impl.getCurrentPhase(),
      'initial phase check'
    ).to.equal(Phase.InputAccumulation);
  });

  ///***test function claim()***///
  it("calling claim() should revert if input duration has not yet past", async ()=> {
    await expect(
      descartesV2Impl.claim(ethers.utils.formatBytes32String("hello")),
      'phase incorrect because inputDuration not over'
    ).to.be.revertedWith('Phase != AwaitingConsensus');

    await timeout((inputDuration/2)*1000);
    await expect(
      descartesV2Impl.claim(ethers.utils.formatBytes32String("hello")),
      'phase incorrect because inputDuration not over'
    ).to.be.revertedWith('Phase != AwaitingConsensus');
  });

  it("should claim() and enter into AwaitingConsensus phase", async ()=>{
    await timeout((inputDuration+1)*1000);
    
    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
    expect(
      await descartesV2Impl.getCurrentPhase(),
      'current phase should be updated to AwaitingConsensus'
    ).to.equal(Phase.AwaitingConsensus);
  });

  it("should claim() and enter into InputAccumulation phase", async ()=>{
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
    await descartesV2Impl.connect(signers[1]).claim(ethers.utils.formatBytes32String("hello"));
    await descartesV2Impl.connect(signers[2]).claim(ethers.utils.formatBytes32String("hello"));

    await mockValidatorManager.mock.onClaim.returns(Result.Consensus,
      [ethers.utils.formatBytes32String("hello"), ethers.utils.formatBytes32String("\0")],
      [await signers[0].getAddress(), address_zero]
    );
    await mockValidatorManager.mock.onNewEpoch.returns(ethers.utils.formatBytes32String("hello"));
    await mockOutput.mock.onNewEpoch.returns();
    await mockInput.mock.onNewEpoch.returns();
    await descartesV2Impl.connect(signers[3]).claim(ethers.utils.formatBytes32String("hello"));
    
    expect(
      await descartesV2Impl.getCurrentPhase(),
      'current phase should be updated to InputAccumulation'
    ).to.equal(Phase.InputAccumulation);
  });

  it("should claim() and enter into AwaitingDispute phase", async ()=>{
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

    await mockValidatorManager.mock.onClaim.returns(Result.Conflict,
      [ethers.utils.formatBytes32String("hello"), ethers.utils.formatBytes32String("halo")],
      [await signers[0].getAddress(), await signers[1].getAddress()]
    );
    await mockDisputeManager.mock.initiateDispute.returns();
    await descartesV2Impl.connect(signers[1]).claim(ethers.utils.formatBytes32String("halo"));
    
    expect(
      await descartesV2Impl.getCurrentPhase(),
      'current phase should be updated to AwaitingDispute'
    ).to.equal(Phase.AwaitingDispute);
  });

  it("two different claim() will enter into AwaitingDispute phase, should revert if there are more claims", async ()=>{
    ///make two different claims///
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

    await mockValidatorManager.mock.onClaim.returns(Result.Conflict,
      [ethers.utils.formatBytes32String("hello"), ethers.utils.formatBytes32String("halo")],
      [await signers[0].getAddress(), await signers[1].getAddress()]
    );
    await mockDisputeManager.mock.initiateDispute.returns();
    await descartesV2Impl.connect(signers[1]).claim(ethers.utils.formatBytes32String("halo"));
    ///END: make two different claims///

    await expect(
      descartesV2Impl.connect(signers[2]).claim(ethers.utils.formatBytes32String("lol")),
      'phase is AwaitingDispute. should revert'
    ).to.be.revertedWith('Phase != AwaitingConsensus');
  });

  ///***test function finalizeEpoch()***///
  it("finalizeEpoch(): should revert if currentPhase is InputAccumulation", async ()=> {
    await expect(
      descartesV2Impl.finalizeEpoch(),
      'phase incorrect'
    ).to.be.revertedWith('Phase != Awaiting Consensus');
  });

  it("finalizeEpoch(): should revert if currentPhase is AwaitingDispute", async ()=> {
    ///make two different claims///
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

    await mockValidatorManager.mock.onClaim.returns(Result.Conflict,
      [ethers.utils.formatBytes32String("hello"), ethers.utils.formatBytes32String("halo")],
      [await signers[0].getAddress(), await signers[1].getAddress()]
    );
    await mockDisputeManager.mock.initiateDispute.returns();
    await descartesV2Impl.connect(signers[1]).claim(ethers.utils.formatBytes32String("halo"));
    ///END: make two different claims///
    
    await expect(
      descartesV2Impl.finalizeEpoch(),
      'phase incorrect'
    ).to.be.revertedWith('Phase != Awaiting Consensus');
  });

  it("finalizeEpoch(): should revert if challengePeriod is not over", async ()=> {
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

    await expect(
      descartesV2Impl.finalizeEpoch(),
      'Challenge period is not over'
    ).to.be.revertedWith('Challenge period is not over');
  });

  it("finalizeEpoch(): should revert if the current claim is null", async ()=> {
    let currentClaim = ethers.utils.formatBytes32String("\0");
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(currentClaim);

    await timeout((challengePeriod+1)*1000);
    await mockValidatorManager.mock.getCurrentClaim.returns(currentClaim);
    await expect(
      descartesV2Impl.finalizeEpoch(),
      'current claim is null'
    ).to.be.revertedWith('No Claim to be finalized');
  });

  it("after finalizeEpoch(), current phase should be InputAccumulation", async ()=> {
    let currentClaim = ethers.utils.formatBytes32String("hello");
    await timeout((inputDuration+1)*1000);

    await mockValidatorManager.mock.onClaim.returns(Result.NoConflict,
      [ethers.utils.formatBytes32String("\0"), ethers.utils.formatBytes32String("\0")],
      [address_zero, address_zero]
    );
    await descartesV2Impl.claim(currentClaim);

    await timeout((challengePeriod+1)*1000);
    await mockValidatorManager.mock.getCurrentClaim.returns(currentClaim);
    await mockValidatorManager.mock.onNewEpoch.returns(ethers.utils.formatBytes32String("hello"));
    await mockOutput.mock.onNewEpoch.returns();
    await mockInput.mock.onNewEpoch.returns();

    await descartesV2Impl.finalizeEpoch();

    expect(
      await descartesV2Impl.getCurrentPhase(),
      'final phase check'
    ).to.equal(Phase.InputAccumulation);
  });

  ///***test function notifyInput()***///
  it("only input contract can call notifyInput()", async ()=> {
    await expect(
      descartesV2Impl.notifyInput(),
      'msg.sender != input contract'
    ).to.be.revertedWith('msg.sender != input contract');
  });

  ///***test function resolveDispute()***///
  it("only DisputeManager contract can call resolveDispute()", async ()=> {
    await expect(
      descartesV2Impl.resolveDispute(await signers[0].getAddress(), await signers[1].getAddress(), ethers.utils.formatBytes32String("hello")),
      'msg.sender != dispute manager contract'
    ).to.be.revertedWith('msg.sender != dispute manager contract');
  });

});

