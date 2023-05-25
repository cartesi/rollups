---
name: ⬆️  Dependency bump
about: Checklist for bumping dependencies
title: ''
labels: chore
assignees: ''
---

## 📚 Context

On-chain or Off-chain?

## 📈 Subtasks

- [ ] Update major versions in `Cargo.toml` and/or `packages.json`.
- [ ] If an update requires major work, create the corresponding issue.
- [ ] Update the dependencies in the lock file (`Cargo.lock` and/or `yarn.lock`).
- [ ] Verify whether everything is working as expected.
