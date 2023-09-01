FROM cartesi/machine-emulator:main

USER 0
RUN apt-get -y update; apt-get -y install curl git; apt-get install -y procps
RUN curl -sSL https://github.com/foundry-rs/foundry/releases/download/nightly/foundry_nightly_linux_$(dpkg --print-architecture).tar.gz | \
    tar -zx -C /usr/local/bin

ADD foundry.toml ./project/
ADD lib ./project/lib/
ADD src ./project/src/
WORKDIR "./project"
RUN forge --version
RUN forge build

ADD ./offchain/ ./offchain/
RUN chmod +x ./offchain/entrypoint.lua
ENTRYPOINT ["./offchain/entrypoint.lua"]
