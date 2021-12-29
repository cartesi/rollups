# Cartesi Rollups

This repository contains the on-chain and off-chain pieces that are used to deploy, launch and interact with Cartesi Rollups DApps. The code presented here is work in progress, continuously being improved and updated.

# Table of contents
- [Documentation](#documentation)
- [Installation](#installation)
- [Talk with us](#talk-with-us)
- [Contributing](#contributing)
- [License](#license)

# Documentation
Several articles were written about the code presented here:
- [Cartesi Rollups - Scalable Smart Contracts Built with mainstream software stacks](https://medium.com/cartesi/scalable-smart-contracts-on-ethereum-built-with-mainstream-software-stacks-8ad6f8f17997)
- [Rollups On-Chain - Tackling Social Scalability](https://medium.com/cartesi/rollups-on-chain-d749744a9cb3)
- [State Fold](https://medium.com/cartesi/state-fold-cfe5f4d79639)
- [Transaction Manager](https://medium.com/cartesi/cartesi-rollups-rollout-transaction-manager-4a49af15d6b9)

# On-chain Rollups:
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

## Voucher

A voucher is a combination of a target address and a payload in bytes. It is used by the off-chain machine to respond and interact with Layer-1 smart contracts. When vouchers get executed they’ll simply send a message to the target address with the payload as a parameter. Therefore, vouchers can be anything ranging from providing liquidity in a DeFi protocol to withdrawing funds from the Portal. Each input can generate a number of vouchers that will be available for execution once the epoch containing them is finalized.
While the Output contract is also indifferent to the content of the voucher being executed, it enforces some sanity checks before allowing its execution: Vouchers are unique and can only be successfully executed once. Vouchers are executed asynchronously and don’t require an access check. The order of execution is not enforced — as long as vouchers are contained in a finalized epoch and were not executed before, the contract will allow their execution by anyone. The Output module ensures, however, that only vouchers suggested by the off-chain machine and finalized on-chain can be executed. It does so by requiring a validity proof to be sent with the execute call.

## Validator Manager

The Validator Manager module was created to help DApps manage their claims, claim permissions, and punishments for bad behavior. Initially, our suggested implementation for this module includes the following characteristics: the set of payable validators is defined in construction time, validators send a claim for every epoch and those that lose a dispute are kicked off the validators set. Cartesi Rollups manager receives claims and redirects them to the Validator Manager. When receiving a claim, the validator manager checks which other claims have arrived at that epoch and returns the information Cartesi Rollups Manager needs to continue. The module can respond to received claims in one of the following ways:
- If the sender is not a validator or the claim is invalid, the transaction reverts.
- If the claim is valid, doesn’t disagree with any of the other claims in the epoch, and does not generate consensus, it returns a “No Conflict” response.
- If the claim is valid but disagrees with another claim for that epoch, it warns Cartesi Rollups Manager that a conflict is happening and what are the conflicting claims and claimants involved. When that dispute is resolved the validator manager module gets notified so it can deal however it likes with the validators involved. In our initially suggested implementation, the loser of a dispute gets removed from the validator set.
- If the claim is valid, agrees with all other claims in that epoch, and is the last claim to be received, it lets Cartesi Rollups know that consensus was reached. This allows the rollups DApp to finalize the epoch and allow for the execution of its vouchers.
Regardless of what the name might suggest, validators do not interact with this module at all.

## Dispute Resolution

Disputes occur when two validators claim different state updates to the same epoch. Because of the deterministic nature of our virtual machine and the fact that the inputs that constitute an epoch are agreed upon beforehand, conflicting claims imply dishonest behavior. When a conflict occurs, the module that mediates the interactions between both validators is the dispute resolution.
The code for rollups dispute resolution is not being published yet - but a big part of it is available on the Cartesi Rollups SDK, using the [Arbitration dlib] (https://github.com/cartesi/arbitration-dlib/)

# Off-chain Rollups
The Rollups machine and the smart contracts live in fundamentally different environments. This creates the need for a middleware that manages and controls the communication between the blockchain and the machine.
As such, the middleware is responsible for first reading data from our smart contracts, then sending them to the machine to be processed, and finally publishing their results back to the blockchain.
The middleware can be used by anyone who's interested in the rollups state of affairs. We divide interested users into two roles, which run different types of nodes: readers and validators. Reader nodes are only interested in advancing their off-chain machine. They consume information from the blockchain but do not bother to enforce state updates, trusting that validators will ensure the validity of all on-chain state updates. Validators, on the other hand, have more responsibility: they not only watch the blockchain but also fight to ensure that the blockchain won't accept that which didn't happen.

# Installation

The Cartesi Rollups infrastructure can be executed in 2 modes:
1. As a **production environment** that provides a Cartesi Machine where the DApp back-end logic will run after being cross-compiled to the RISC-V architecture. Please check our [examples repository](https://github.com/cartesi/rollups-examples) to learn how to use Cartesi Rolllups usage.
2. As a **host environment** that provides the very same HTTP API as the regular one, mimicking the behavior of the actual layer-1 and layer-2 components. This way, the Cartesi Rollups infrastructure can make HTTP requests to a native back-end running on localhost. This allows the developer to run and debug them using familiar tools, such as an IDE. To execute the Cartesi Rollups as a host environment execute the following commands:


        # Clone repo
        git clone --recurse-submodules git@github.com:cartesi/rollups.git
        cd rollups

        # Run docker compose up
        docker-compose up

        # Wait for the hardhat container to start and create the rollup onchain infrastructure:
        docker exec rollups_hardhat_1 npx hardhat --network localhost rollups:create --export dapp.json

        # You'll know this was created when this command shows you the following reply:
        Rollups Impl address: 0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
        Rollups Impl getCurrentEpoch: 0
        Rollups accumulation start: 1640643011
        Input address 0x9bd03768a7DCc129555dE410FF8E85528A4F88b5
        Output address 0x440C0fCDC317D69606eabc35C0F676D1a8251Ee1
        Ether Portal address 0x8A791620dd6260079BF849Dc5567aDC3F2FdC318
        ERC20 Portal address 0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6

        # To send an input, open a new terminal window and run:
        docker exec rollups_hardhat_1 npx hardhat --network localhost input:addInput --input "0xdeadbeef"

        # You'll know the input was accepted when the terminal shows you something like following reply:
        Added input '0xdeadbeef' to epoch '0' (timestamp: 1640643170, signer: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266, tx: 0x31d7e9e810d8702196623837b8512097786b544a4b5ffb52f693b9ff7d424147)

        # In order to verify the vouchers or notices generated by your inputs, run the command:
        curl -v http://localhost:4000/graphql -H 'Content-Type: application/json' -d '{ "query" : "query getNotice { GetNotice( query: { session_id: \"default_rollups_id\", epoch_index: \"0\", input_index: \"0\" } ) { session_id epoch_index input_index notice_index payload keccak_in_notice_hashes { id target_hash target_address }} }" }'

        # The response should be something like:
        *   Trying 127.0.0.1:4000...
        * TCP_NODELAY set
        * Connected to localhost (127.0.0.1) port 4000 (#0)
        > POST /graphql HTTP/1.1
        > Host: localhost:4000
        > User-Agent: curl/7.68.0
        > Accept: */*
        > Content-Type: application/json
        > Content-Length: 251
        > 
        * upload completely sent off: 251 out of 251 bytes
        * Mark bundle as not supporting multiuse
        < HTTP/1.1 200 OK
        < X-Powered-By: Express
        < Access-Control-Allow-Origin: *
        < Content-Type: application/json; charset=utf-8
        < Content-Lengt

        # To stop the containers
        # first end the process with ctrl + c
        # then remove volumes
        docker-compose down --volumes

       # The HTTP API uses two different ports, to receive and send information, they're respectively: 5002 and 5003

# Talk with us
If you’re interested in developing with Cartesi, working with the team, or hanging out in our community, don’t forget to [join us on Discord and follow along](https://discordapp.com/invite/Pt2NrnS).

Want to stay up to date? Make sure to join our [announcements channel on Telegram](https://t.me/CartesiAnnouncements) or [follow our Twitter](https://twitter.com/cartesiproject).

# Contributing

Thank you for your interest in Cartesi! Head over to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to sign our Contributors Agreement and get started with Cartesi!

Please note we have a [Code of Conduct](CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

# License

Note: This component currently has dependencies that are licensed under the GNU GPL, version 3, and so you should treat this component as a whole as being under the GPL version 3. But all Cartesi-written code in this component is licensed under the Apache License, version 2, or a compatible permissive license, and can be used independently under the Apache v2 license. After this component is rewritten, the entire component will be released under the Apache v2 license.
The arbitration d-lib repository and all contributions are licensed under
[GPL 3](https://www.gnu.org/licenses/gpl-3.0.en.html). Please review our [COPYING](COPYING) file.
