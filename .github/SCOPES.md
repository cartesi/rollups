## Scopes

The scopes on this repository has the following rules:

If you're changing the `offchain`, use the name of the crate of the service your changes are affecting. If it's not one of the main services, but an auxiliary crate, use the generic `offchain`.

The following is the list of supported scopes for the offchain:

* `advance-runner`
* `dispatcher`
* `graphql-server`
* `indexer`
* `inspect-server`
* `state-server`
* `offchain`

If you're changing the `onchain`, use `contracts` for `rollups/contracts`; Use `deploy` for `rollups/deploy`; Use `arbitration` for `rollups-arbitration`; Use `cli` for `rollups-cli`

The following is the list of supported scopes for the onchain:

* `contracts`
* `deploy`
* `arbitration`
* `cli`
