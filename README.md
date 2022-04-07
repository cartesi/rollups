# Cartesi Rollups

This repository contains the on-chain and off-chain pieces that are used to deploy, launch and interact with Cartesi Rollups DApps. The code presented here is work in progress, continuously being improved and updated.

# Table of contents
- [Documentation](#documentation)
- [Experimenting](#experimenting)
- [Talk with us](#talk-with-us)
- [Contributing](#contributing)
- [Setting up](#setting-up)
- [Testing](#testing)
- [License](#license)

## Documentation
Several articles were written about the code presented here:
- [Cartesi Rollups - Scalable Smart Contracts Built with mainstream software stacks](https://medium.com/cartesi/scalable-smart-contracts-on-ethereum-built-with-mainstream-software-stacks-8ad6f8f17997)
- [Rollups On-Chain - Tackling Social Scalability](https://medium.com/cartesi/rollups-on-chain-d749744a9cb3)
- [State Fold](https://medium.com/cartesi/state-fold-cfe5f4d79639)
- [Transaction Manager](https://medium.com/cartesi/cartesi-rollups-rollout-transaction-manager-4a49af15d6b9)

## On-chain Rollups:
Designed to mediate the relationship between the off-chain components with other smart contracts and externally owned accounts. It is composed by several modules, each with clear responsibilities and well-defined interfaces. The modules are the following:

## Cartesi Rollups Manager

The Cartesi Rollups Manager is responsible for synchronicity between the modules. It defines the duration of the different phases and notifies the other modules of any phase change. Among others, the responsibilities of this module are:
- Define for which epoch, depending on the current state and deadlines, an input is destined to.
- Receive and forward claims to the Validator Manager.
- Forward disputes to the Dispute Resolution module.
- Forward the result of disputes to the Validator Manager.
- Forward the summary of finalized outputs (vouchers and notices) to the Output module.
- Notify other modules of phase changes.

## Input

The on-chain contracts often have two concurrent epochs: a sealed but unfinalized epoch and an accumulating epoch. The input contract keeps one inbox for each of those epochs, switching between them depending on the Cartesi Rollups Manager's notifications. For anyone to be able to synchronize the machine from its beginning without needing to trust a data provider, the full content of inputs is always present in calldata. In storage, which needs to be used in a more parsimonious way, we keep a single hash for each input of an active epoch. This input hash summarizes both the input itself and its metadata — sender’s address and time of reception. Notice that this input implementation is permissionless, the permission layer is delegated to the off-chain machine which will, for example, judge if a sender is allowed to do what their input wants to do.

## Portal

Portal, as the name suggests, is used to teleport assets from the Ethereum blockchain to DApps running on Cartesi Rollups. Once deposited, those Layer-1 assets gain a representation in Layer-2 and are owned, there, by whomever the depositor assigned them to. After being teleported, Layer-2 assets can be moved around in a significantly cheaper way, using simple inputs that are understood by the Linux logic. When an asset is deposited, the Portal contract sends an input to the DApp’s inbox, describing the type of asset, amount, receivers, and some data the depositor might want the DApp to read. This allows deposits and instructions to be sent as a single Layer-1 interaction.
One could think of the Portal as a bank account, owned by the off-chain machine. Anyone can deposit assets there but only the DApp — through its Output contract — can decide on withdrawals. The withdrawal process is quite simple from a user perspective. They send an input requesting a withdrawal, which gets processed and interpreted off-chain. If everything is correct, the machine creates a voucher destined to the Portal contract, ordering and finalizing that withdrawal request.
Currently, we support the following types of assets:
- [Ether](https://ethereum.org/en/eth/) (ETH)
- [ERC-20](https://ethereum.org/en/developers/docs/standards/tokens/erc-20/)
- [ERC-721](https://ethereum.org/en/developers/docs/standards/tokens/erc-721/) (NFTs)

## Voucher

A voucher is a combination of a target address and a payload in bytes. It is used by the off-chain machine to respond and interact with Layer-1 smart contracts. When vouchers get executed they’ll simply send a message to the target address with the payload as a parameter. Therefore, vouchers can be anything ranging from providing liquidity in a DeFi protocol to withdrawing funds from the Portal. Each input can generate a number of vouchers that will be available for execution once the epoch containing them is finalized.
While the Output contract is also indifferent to the content of the voucher being executed, it enforces some sanity checks before allowing its execution: Vouchers are unique and can only be successfully executed once. Vouchers are executed asynchronously and don’t require an access check. The order of execution is not enforced — as long as vouchers are contained in a finalized epoch and were not executed before, the contract will allow their execution by anyone. The Output module ensures, however, that only vouchers suggested by the off-chain machine and finalized on-chain can be executed. It does so by requiring a validity proof to be sent with the execute call.

## Notices

Notices are informational statements that can be proved on L1 by other smart contracts. They're emitted by the off-chain machine and contain a payload, in bytes. DApp developers are free to explore different use cases for notices, their generality and negligible cost of emission makes them a powerful tool to assist integration between L2 DApps and L1 smart contracts or even other L2 DApps. Similarly to vouchers, notices can only be proved once they've been finalized on-chain and if they're accompanied by a validity proof. A chess DApp could, for example, emit a notice informing the underlying blockchain of the winner of a tournament - while that information is not necessarily "actionable", it could be used by other applications for different purposes.

## Validator Manager

The Validator Manager module was created to help DApps manage their claims, claim permissions, and punishments for bad behavior. Initially, our suggested implementation for this module includes the following characteristics: the set of payable validators is defined in construction time, validators send a claim for every epoch and those that lose a dispute are kicked off the validators set. Cartesi Rollups manager receives claims and redirects them to the Validator Manager. When receiving a claim, the validator manager checks which other claims have arrived at that epoch and returns the information Cartesi Rollups Manager needs to continue. The module can respond to received claims in one of the following ways:
- If the sender is not a validator or the claim is invalid, the transaction reverts.
- If the claim is valid, doesn’t disagree with any of the other claims in the epoch, and does not generate consensus, it returns a “No Conflict” response.
- If the claim is valid but disagrees with another claim for that epoch, it warns Cartesi Rollups Manager that a conflict is happening and what are the conflicting claims and claimants involved. When that dispute is resolved the validator manager module gets notified so it can deal however it likes with the validators involved. In our initially suggested implementation, the loser of a dispute gets removed from the validator set.
- If the claim is valid, agrees with all other claims in that epoch, and is the last claim to be received, it lets Cartesi Rollups know that consensus was reached. This allows the rollups DApp to finalize the epoch and allow for the execution of its vouchers.
Regardless of what the name might suggest, validators do not interact with this module at all.

## Fee Manager and Bank

The Fee Manager facet and Bank contract aim to create an economical incentive for validators to run a given DApp. When deploying their application, deployers define a fee to be paid on a claim by claim basis and an owner for the fee manager facet, who has the option to change the fee value, if needed. The value chosen, in CTSI, is reserved for a validator every time a claim is made and can be withdrawn by them whenever convenient.
While the Fee Manager controls the fee value and the amount of claims made by each validator, the Bank stores the CTSI tokens - on a separate contract - that will be distributed to them. The code does not enforce a way for the Bank to be funded, DApps and communities are free to choose their preferred methods. Options include, but are not limited to: direct transfer, charging per input or creating a tax system on top of the portal. For convenience we added a `FundBank` hardhat task which transfers money from the signer to a DApp's bank.
Validators configure their nodes to either be altruistic or to have a minimum retainer. Before processing inputs and submitting claims, if they're not altruistic, they check if two conditions are true: the fee paid by the DApp is bigger than their required retainer and the DApp's Bank has enough funds to pay them. If they're altruistic they'll work regardless of financial compensation.

## Dispute Resolution

Disputes occur when two validators claim different state updates to the same epoch. Because of the deterministic nature of our virtual machine and the fact that the inputs that constitute an epoch are agreed upon beforehand, conflicting claims imply dishonest behavior. When a conflict occurs, the module that mediates the interactions between both validators is the dispute resolution.
The code for rollups dispute resolution is not being published yet - but a big part of it is available on the Cartesi Rollups SDK, using the [Arbitration dlib] (https://github.com/cartesi/arbitration-dlib/)

## Off-chain Rollups
The Rollups machine and the smart contracts live in fundamentally different environments. This creates the need for a middleware that manages and controls the communication between the blockchain and the machine.
As such, the middleware is responsible for first reading data from our smart contracts, then sending them to the machine to be processed, and finally publishing their results back to the blockchain.
The middleware can be used by anyone who's interested in the rollups state of affairs. We divide interested users into two roles, which run different types of nodes: readers and validators. Reader nodes are only interested in advancing their off-chain machine. They consume information from the blockchain but do not bother to enforce state updates, trusting that validators will ensure the validity of all on-chain state updates. Validators, on the other hand, have more responsibility: they not only watch the blockchain but also fight to ensure that the blockchain won't accept that which didn't happen.

## Experimenting
To get a taste on how to use Cartesi to develop your DApp, check the following resources:
See Cartesi Rollups in action with the [Simple Echo Example](https://github.com/cartesi/rollups-examples/tree/main/echo).
To have a glimpse on how to develop your DApp locally using your favorite IDE and tools check our [Host Environment](https://github.com/cartesi/rollups-examples/tree/main/host) repo.


## Talk with us
If you’re interested in developing with Cartesi, working with the team, or hanging out in our community, don’t forget to [join us on Discord and follow along](https://discordapp.com/invite/Pt2NrnS).

Want to stay up to date? Make sure to join our [announcements channel on Telegram](https://t.me/CartesiAnnouncements) or [follow our Twitter](https://twitter.com/cartesiproject).

## Contributing

Thank you for your interest in Cartesi! Head over to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to sign our Contributors Agreement and get started with Cartesi!

Please note we have a [Code of Conduct](CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## Setting up

### Initialize submodules recursively

In order to also clone submodules like `grpc-interfaces` and `state-fold`, you need to run the following command.

```sh
git submodule update --init --recursive
```

### Compile on-chain code

The on-chain part is mainly written in Solidity and Typescript. For that, you'll need `yarn` to install dependencies and to run build scripts.

```sh
cd onchain
yarn
cd rollups
yarn build
```

### Compile off-chain code

The off-chain code is written in Rust. For that, you'll need `cargo`. See the [Rust documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html) for instructions on how to install `cargo` on your system.

```sh
cd offchain
cargo build
```

## Testing

Once you've setup the repository, you can test the different pieces that compose Cartesi Rollups individually.

### Testing the on-chain code

```sh
cd onchain/rollups
yarn test
```

### Testing the delegate server

In order to generate the `.proto` files for Typescript, run the following commands once.

```sh
cd onchain/rollups
yarn test:protoc
```

Now you can run the delegate test bench script.

```sh
./scripts/delegate_tests.sh
```

# License

Note: This component currently has dependencies that are licensed under the GNU GPL, version 3, and so you should treat this component as a whole as being under the GPL version 3. But all Cartesi-written code in this component is licensed under the Apache License, version 2, or a compatible permissive license, and can be used independently under the Apache v2 license. After this component is rewritten, the entire component will be released under the Apache v2 license.
The arbitration d-lib repository and all contributions are licensed under
[GPL 3](https://www.gnu.org/licenses/gpl-3.0.en.html). Please review our [COPYING](COPYING) file.
