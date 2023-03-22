---
name: Smart Contract Audit
about: Template for auditing smart contracts
title: ''
labels: ''
assignees: ''

---

## Vulnerabilities

Listed below are some well documented vulnerabilities that affect smart contracts.
You can find many others from the list of references.

### External calls
- [ ] Unchecked Call Return Value [(SWC-104)](https://swcregistry.io/docs/SWC-104)
- [ ] Re-entrancy attacks [(SWC-107)](https://swcregistry.io/docs/SWC-107)
- [ ] Delegate Call to Untrusted Callee [(SWC-112)](https://swcregistry.io/docs/SWC-112)
- [ ] Insufficient Gas Griefing [(SWC-126)](https://swcregistry.io/docs/SWC-126)
- [ ] Message call with hardcoded gas amount [(SWC-134)](https://swcregistry.io/docs/SWC-134)
- [ ] Untrustworthy Data Feeds [(Al-Breiki et al., 2020)](https://doi.org/10.1109/ACCESS.2021.3140091)

### Denial-of-Service attacks
- [ ] Failed Call [(SWC-113)](https://swcregistry.io/docs/SWC-113)
- [ ] Block Gas Limit [(SWC-128)](https://swcregistry.io/docs/SWC-128)
- [ ] Unexpected Ether balance [(SWC-132)](https://swcregistry.io/docs/SWC-132)

### Miner attacks
- [ ] Transaction Order Dependence [(SWC-114)](https://swcregistry.io/docs/SWC-114)
- [ ] Weak Sources of Randomness from Chain Attributes [(SWC-120)](https://swcregistry.io/docs/SWC-120)

### Authorization
- [ ] Unprotected Ether Withdrawal [(SWC-105)](https://swcregistry.io/docs/SWC-105)
- [ ] Unprotected Self-Destruct [(SWC-106)](https://swcregistry.io/docs/SWC-106)
- [ ] State Variable Default Visibility [(SWC-108)](https://swcregistry.io/docs/SWC-108)
- [ ] Authorization through `tx.origin` [(SWC-115)](https://swcregistry.io/docs/SWC-115)
- [ ] Signature Malleability [(SWC-117)](https://swcregistry.io/docs/SWC-117)
- [ ] Missing Protection against Signature Replay Attacks [(SWC-121)](https://swcregistry.io/docs/SWC-121)
- [ ] Lack of Proper Signature Verification [(SWC-122)](https://swcregistry.io/docs/SWC-122)

### Programming errors
- [ ] Integer Underflow and Overflow [(SWC-101)](https://swcregistry.io/docs/SWC-101)
- [ ] Use of Deprecated Solidity Functions [(SWC-111)](https://swcregistry.io/docs/SWC-111)
- [ ] Block values as a proxy for time [(SWC-116)](https://swcregistry.io/docs/SWC-116)
- [ ] Shadowing State Variables [(SWC-119)](https://swcregistry.io/docs/SWC-119)
- [ ] Requirement Violation [(SWC-123)](https://swcregistry.io/docs/SWC-123)
- [ ] Write to Arbitrary Storage Location [(SWC-124)](https://swcregistry.io/docs/SWC-124)
- [ ] Incorrect Inheritance Order [(SWC-125)](https://swcregistry.io/docs/SWC-125)
- [ ] Arbitrary Jump with Function Type Variable [(SWC-127)](https://swcregistry.io/docs/SWC-127)
- [ ] Typographical Error [(SWC-129)](https://swcregistry.io/docs/SWC-129)
- [ ] Presence of unused variables [(SWC-131)](https://swcregistry.io/docs/SWC-131)
- [ ] Hash Collision With Multiple Variable-Length Arguments [(SWC-133)](https://swcregistry.io/docs/SWC-133)
- [ ] Code With No Effects [(SWC-135)](https://swcregistry.io/docs/SWC-135)
- [ ] Unencrypted Private Data On-Chain [(SWC-136)](https://swcregistry.io/docs/SWC-136)

### Compiler
- [ ] Outdated Compiler Version [(SWC-102)](https://swcregistry.io/docs/SWC-102)
- [ ] Floating Pragma [(SWC-103)](https://swcregistry.io/docs/SWC-103)

## References

Smart Contract Weakness Classification and Test Cases. https://swcregistry.io/

Kushwaha, S. S., Joshi, S., Singh, D., Kaur, M., & Lee, H.-N. (2022). Systematic Review of Security Vulnerabilities in Ethereum Blockchain Smart Contract. IEEE Access, 10, 6605–6621. https://doi.org/10.1109/ACCESS.2021.3140091

Al-Breiki, H., Rehman, M. H. U., Salah, K., & Svetinovic, D. (2020). Trustworthy Blockchain Oracles: Review, Comparison, and Open Research Challenges. IEEE Access, 8, 85675–85685. https://doi.org/10.1109/ACCESS.2020.2992698
