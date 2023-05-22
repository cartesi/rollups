---
name: ⬆️ Update Dependencies
about: Checklist when updating dependencies
title: ''
labels: chore
assignees: ''

---

## 📚 Context

On-chain or Off-chain?

## 📈 Subtasks

- [ ] Update major versions in `cargo.toml` and/or `packages.json`.
- [ ] If an update requires major work, create the corresponding issue.
- [ ] Update the dependencies in the lock file (`cargo.lock` and/or `yarn.lock`).
- [ ] Verify whether everything is working as expected.
