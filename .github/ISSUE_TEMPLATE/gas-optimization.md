---
name: Gas Optimization
about: Template for solidity smart contract gas optimization
title: ''
labels: ''
assignees: ''

---

## Steps
- [ ] configure and run the [HardHat profiler](https://www.npmjs.com/package/hardhat-gas-reporter)
- [ ] check the boxes in the checklist if they are applicable. Leave out those that are not applicable. It would be helpful to record the gas savings amount for modification steps.
- [ ] create a PR to compare the original/current gas report, display the amount of gas savings of modification steps, discuss and comment.
- [ ] you can use [this Python script](https://gist.github.com/guidanoli/b7566f54c437e0b0f28c4554d151286f) to ease the job of comparing Hardhat gas reports. It takes two Hardhat gas reports (before and after some change) and outputs a Markdown table comparing them. Other options are available.
- [ ] (optional) Learn more about [gas consumption components of deploying a contract](https://ethereum.stackexchange.com/questions/35539/what-is-the-real-price-of-deploying-a-contract-on-the-mainnet/37898), [more detailed analysis](https://hackernoon.com/costs-of-a-real-world-ethereum-contract-2033511b3214), and how to [reduce contract bytecode](https://medium.com/daox/avoiding-out-of-gas-error-in-large-ethereum-smart-contracts-18961b1fc0c6)

## Checklist
- [ ] 1. Pack storage variables
- [ ] 2. uint8 is not always cheaper than uint256
- [ ] 3. Mappings are cheaper than Arrays
- [ ] 4. Elements in Memory and Call Data cannot be packed
- [ ] 5. Use bytes32 rather than string/bytes
- [ ] 6. Make fewer external calls
- [ ] 7. Use external function modifier
- [ ] 8. Delete variables that you don’t need
- [ ] 9. Use Short Circuiting rules to your advantage
- [ ] 10. Avoid changing storage data
- [ ] 11. Function modifiers can be inefficient
- [ ] 12. Booleans use 8 bits while you only need 1 bit
- [ ] 13. Use libraries to save some bytecode
- [ ] 14. No need to initialize variables with default values
- [ ] 15. Use short reason strings
- [ ] 16. Avoid repetitive checks
- [ ] 17. Make use of single line swaps
- [ ] 18. Use events to store data that is not required on-chain
- [ ] 19. Make proper use of the optimizer
- [ ] 20. Using fewer functions can be helpful
- [ ] 21. Calling internal functions is cheaper
- [ ] 22. Using proxy patterns for mass deployment
- [ ] 23. General logic/programming optimization

Please refer to [this article](https://mudit.blog/solidity-gas-optimization-tips/) and [this article](https://blog.polymath.network/solidity-tips-and-tricks-to-save-gas-and-reduce-bytecode-size-c44580b218e6) for details regarding the checkpoints.
