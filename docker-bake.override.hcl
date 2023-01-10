
variable "TAG" {
  default = "devel"
}

variable "DOCKER_ORGANIZATION" {
  default = "cartesi"
}

target "state-server" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-state-server:${TAG}"]
}

target "dispatcher" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-dispatcher:${TAG}"]
}

target "indexer" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-indexer:${TAG}"]
}

target "inspect-server" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-inspect-server:${TAG}"]
}

target "server-manager-broker-proxy" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-server-manager-broker-proxy:${TAG}"]
}

target "reader" {
  tags = ["${DOCKER_ORGANIZATION}/query-server:${TAG}"]
}

target "hardhat" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-hardhat:${TAG}"]
}

target "cli" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-cli:${TAG}"]
}

target "deployments" {
  tags = ["${DOCKER_ORGANIZATION}/rollups-deployments:${TAG}"]
}
