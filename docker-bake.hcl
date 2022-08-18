
target "docker-metadata-action" {}

group "default" {
  targets = ["state-server", "dispatcher", "indexer", "inspect-server", "reader", "hardhat", "rollups-cli"]
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

target "reader" {
  inherits   = ["docker-metadata-action"]
  dockerfile = "offchain/Dockerfile"
  target     = "reader"
  context    = "."
}

target "hardhat" {
  inherits = ["docker-metadata-action"]
  context  = "./onchain/rollups"
}

target "rollups-cli" {
  inherits   = ["docker-metadata-action"]
  context    = "./onchain"
  dockerfile = "./rollups-cli/Dockerfile"
}
