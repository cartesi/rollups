
target "docker-metadata-action" {}

group "default" {
  targets = ["state-server", "dispatcher", "inspect-server", "graphql-server", "server-manager-broker-proxy", "hardhat", "cli"]
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

target "server-manager-broker-proxy" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "server_manager_broker_proxy"
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
