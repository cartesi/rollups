
target "docker-metadata-action" {}

group "default" {
  targets = [
    "advance-runner",
    "cli",
    "dispatcher",
    "graphql-server",
    "hardhat",
    "inspect-server",
    "indexer",
    "state-server"
  ]
}

target "deps" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "builder"
  context    = "."
}

target "state-server" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "state_server"
  context    = "."
}

target "dispatcher" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "dispatcher"
  context    = "."
}

target "indexer" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "indexer"
  context    = "."
}

target "inspect-server" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "inspect_server"
  context    = "."
}

target "graphql-server" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "graphql_server"
  context    = "."
}

target "advance-runner" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "advance_runner"
  context    = "."
}

target "hardhat" {
  inherits = ["docker-metadata-action"]
  context  = "./onchain"
  target   = "hardhat"
}

target "cli" {
  inherits = ["docker-metadata-action"]
  context  = "./onchain"
  target   = "cli"
}

target "deployments" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "deployments"
  context    = "."
  platforms = [
    "linux/amd64",
    "linux/arm64",
    "linux/riscv64"
  ]
}
