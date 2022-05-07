
group "default" {
  targets = ["delegate_server", "dispatcher", "indexer", "reader"]
}

target "delegate_server" {
  dockerfile = "offchain/delegate_server/Dockerfile"
  context    = "."
}

target "dispatcher" {
  dockerfile = "offchain/delegate_server/Dockerfile"
  context    = "."
}

target "indexer" {
  dockerfile = "offchain/indexer/Dockerfile"
  context    = "."
}

target "reader" {
  dockerfile = "Dockerfile"
  context    = "./reader"
}
